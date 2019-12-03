import os
import threading
import time
from os.path import isfile, join
from time import perf_counter

from tx_gen.utils import read_blockchain_parameters, get_official_history, wait_new_block_created, get_logs_summary, \
    get_current_date, delete_folder, sorted_nicely
from tx_gen.addresses import create_address, create_addr_from_sk_key
from tx_gen.transactions import encode_and_post_txs, create_tx_send_funds_acc_to_acc, get_account_details, \
    send_funds_from_acc_to_acc_list, encode_and_post_tx
import pandas as pd

pd.set_option('display.max_columns', None)
pd.set_option('display.max_rows', None)
pd.set_option('display.max_colwidth', -1)


def create_txs(no_of_txs, tx_value, api_url_base, src_add_sk, dst_acc_addr, fee_constant, fee_coefficient,
               fee_certificate, location_offline_tx_files, block_0_hash):
    src_addr = create_addr_from_sk_key(src_add_sk, "account")
    src_addr_counter = get_account_details(src_addr, api_url_base)[0]

    # create the offline transactions
    for tx_counter in range(0, no_of_txs):
        offline_tx = location_offline_tx_files + "tx_" + str(tx_counter)
        witness_out_file = location_offline_tx_files + "witness_tx_" + str(tx_counter)
        witness_secret_file = location_offline_tx_files + "witness_secret_tx_" + str(tx_counter)

        # write the secret key of the faucet/source account inside the witness_secret_file
        wr = open(witness_secret_file, 'w')
        wr.write(src_add_sk)
        wr.close()

        # spending_counter will be incremented for all transactions
        src_spending_counter = src_addr_counter + tx_counter

        # create the offline transactions
        create_tx_send_funds_acc_to_acc(offline_tx, dst_acc_addr, tx_value, src_add_sk, fee_constant,
                                        fee_coefficient, fee_certificate, block_0_hash, witness_out_file,
                                        witness_secret_file, src_spending_counter)

        # remove the witness files (not needed after the transaction was created)
        for fname in os.listdir(location_offline_tx_files):
            if fname.startswith("witness"):
                os.remove(os.path.join(location_offline_tx_files, fname))


def print_stats(no_of_txs, api_url_base_list, dst_addr_list, test_start_date, t1_start, t1_stop, t2_start, t2_stop, t3_stop):
    df_block_dates_latest_epochs = get_official_history(2, f"{api_url_base_list[0]}/v0")
    no_of_blocks_created_during_measurement_duration = \
        df_block_dates_latest_epochs[df_block_dates_latest_epochs['date'].str.contains(test_start_date, na=False)].index[0]

    # Display stats
    dst_addr_balances = []
    for addr in dst_addr_list:
        dst_addr_balance = get_account_details(addr, api_url_base_list[0])[1]
        dst_addr_balances.append(int(dst_addr_balance))

    print("================== Cumulative stats per test ==================")
    print(f"No of generated transactions         : {no_of_txs * len(dst_addr_list)}")
    print(f"No of mined txs (based on dst values): {sum(dst_addr_balances)} --> {dst_addr_balances}")
    print(f"Tx creation speed [txs/sec]          : {round(no_of_txs * len(dst_addr_list) / (t1_stop - t1_start), 2)}")
    print(f"Tx posting speed [txs/sec]           : {round(no_of_txs * len(dst_addr_list) / (t2_stop - t2_start), 2)}")
    print(f"Time for creating transactions [sec] : {round(t1_stop - t1_start, 2)} = {time.strftime('%H:%M:%S', time.gmtime(t1_stop - t1_start))}")
    print(f"Time for sending/posting txs [sec]   : {round(t2_stop - t2_start, 2)} = {time.strftime('%H:%M:%S', time.gmtime(t2_stop - t2_start))}")
    print(f"TPS - measurement duration [sec]     : {round(t3_stop - t2_start, 2)} = {time.strftime('%H:%M:%S', time.gmtime(t3_stop - t2_start))}")
    print(f"Start measurement duration date      : {test_start_date}")
    print(f"No of blocks created (measurement)   : {no_of_blocks_created_during_measurement_duration}")
    print(f"TPS (mined transactions per second)  : {round(sum(dst_addr_balances) / (t3_stop - t2_start), 2)}")

    print("=================== Display the message logs status summary ===================")
    for api_url_base in api_url_base_list:
        get_logs_summary(api_url_base)


def generate_txs(no_of_sources, no_of_txs_per_source, api_url_base_list, faucet_addr_sk):
    """
    Scope:
        - send a specified number of transactions from a number of source addresses to the same number of destinations
        - if there is more than 1 source, the transactions will be created in parallel, per source address
        - if there is more than 1 source, the transactions will posted/sent in parallel, per source address
    Parameters:
        - no_of_sources = number of source address accounts to initiate transactions (no of threads)
        - no_of_txs_per_source = number of transactions to be initiated from each source (from no_of_sources)
            - no_of_txs_per_src_addr = no_of_total_txs / no_of_sources
        - api_url_base_list = list of node rest listen addresses that would be used to send transactions
            - (Ex: ["http://127.0.0.1:3101/api", "http://127.0.0.1:3102/api"])
        - faucet_addr_sk = private key of the address containing the funds for the current test
    Steps:
        1. create the requested number of (source) account addresses - no_of_sources
        2. send funds from the faucet address (faucet_addr_sk) to the source addresses created above
        3. create 1 new address (destination/receiver) for any source address - no_of_sources
        4. for each source address, create the specified number of transactions in a separate folder, per source
        - the transactions will have an incremental counter (starting with the counter value of the source address from
        the beginning of the test)
        - the transactions for each source are created on a different thread
        5. post/sent the offline transactions
        - the transactions for each source are sent on a different thread
        - the transactions are sent on multiple nodes (based on the provided list)
        6. wait for 1 new block to be created (as defined by the Measurement Duration for TPS)
        7. clean up = delete the offline transactions folders
        8. print the cumulative stats for the test
    """
    api_url_base = api_url_base_list[0]
    api_url = f"{api_url_base}/v0"
    offline_tx_directory = "./offline_txs/"
    tx_value = 1

    # Remove the offline transactions folder if it exists
    delete_folder(offline_tx_directory)

    # Read the blockchain parameters
    block_0_hash, max_txs_per_block, slot_duration, slots_per_epoch, fee_constant, fee_coefficient, fee_certificate = \
        read_blockchain_parameters(api_url)

    # Create the source addresses
    print(f"============== Creating future source account address --> {no_of_sources}  ================")
    src_addr_sk_list = []
    src_addr_list = []
    for i in range(0, no_of_sources):
        new_addr_dict = create_address('account')
        addr_sk = new_addr_dict['sk']
        addr = new_addr_dict['addr']
        src_addr_sk_list.append(addr_sk)
        src_addr_list.append(addr)

    print("============== Sending funds from initial account to future Source accounts  ================")
    # Send funds form the provided faucet private key to the source addresses
    faucet_addr = create_addr_from_sk_key(faucet_addr_sk, "account")
    faucet_balance_init = get_account_details(faucet_addr, api_url_base)[1]

    amount_per_src_acc = (tx_value + fee_constant + 2 * fee_coefficient) * (no_of_txs_per_source + 5)
    send_funds_from_acc_to_acc_list(offline_tx_directory, faucet_addr_sk, src_addr_list, amount_per_src_acc,
                                    api_url_base, fee_constant, fee_coefficient, fee_certificate, block_0_hash)

    src_addr_balances = []
    for addr in src_addr_list:
        addr_balance = get_account_details(addr, api_url_base)[1]
        src_addr_balances.append(addr_balance)

    faucet_balance = get_account_details(faucet_addr, api_url_base)[1]

    print("=============================================================================")
    print(f"Node address            : {api_url_base}")
    print(f"Faucet address          : {faucet_addr}")
    print(f"Faucet initial balance  : {faucet_balance_init}")
    print(f"Faucet balance          : {faucet_balance}")
    print(f"Source addresses        : {src_addr_list}")
    print(f"Source balances         : {src_addr_balances}")
    print("=============================================================================")

    # Remove the offline transactions folder if it exists (in order to keep only the desired txs there)
    delete_folder(offline_tx_directory)

    # Create new account addresses - to be used as Destination Addresses for the transactions
    offline_txs_list = []
    offline_txs_locations = []
    dst_addr_list = []
    for i in range(no_of_sources):
        new_addr_dict = create_address('account')
        dst_acc_addr = new_addr_dict["addr"]
        dst_addr_list.append(dst_acc_addr)

    # Create the offline transactions for the test (form 1 Source address to 1 Destination address)
    print(f"================= Creating {no_of_sources * no_of_txs_per_source} offline transactions ================")
    t1_start = perf_counter()
    threads1 = []
    for i in range(no_of_sources):
        print(f"====== Creating {no_of_txs_per_source} transactions for {src_addr_sk_list[i]} ========")
        location_offline_tx_files = offline_tx_directory + src_addr_sk_list[i][-5:] + "/"
        os.makedirs(os.path.dirname(location_offline_tx_files), exist_ok=True)
        offline_txs_locations.append(location_offline_tx_files)

        t = threading.Thread(target=create_txs, args=(no_of_txs_per_source, tx_value, api_url_base,
                                                      src_addr_sk_list[i], dst_addr_list[i], fee_constant,
                                                      fee_coefficient, fee_certificate, location_offline_tx_files,
                                                      block_0_hash,))
        threads1.append(t)
        t.start()

    for t in threads1:
        t.join()

    print('all threads done! (tx creation)')
    t1_stop = perf_counter()

    # Read all created transactions for each source address (ordered 'naturally')
    for x in offline_txs_locations:
        unsorted_txs = [f for f in os.listdir(x) if isfile(join(x, f))]
        sorted_txs = sorted_nicely(unsorted_txs)
        offline_txs_list.append(sorted_txs)

    # Create a list of nodes on which the transactions will be sent (it should have the same length as the no_of_sources)
    node_list = []
    while len(node_list) < len(src_addr_sk_list):
        for node in api_url_base_list:
            node_list.append(node)

    # Post the offline transactions (on different nodes in parallel)
    test_start_date = get_current_date(api_url)
    t2_start = perf_counter()

    threads2 = []
    for i in range(len(src_addr_sk_list)):
        t = threading.Thread(target=encode_and_post_txs, args=(offline_txs_list[i], node_list[i], offline_txs_locations[i],))
        threads2.append(t)
        t.start()

    for t in threads2:
        t.join()

    print('all threads done! (tx post)')
    t2_stop = perf_counter()

    # Wait for 1 new block to be created - as part of the measurement duration definition
    wait_new_block_created(200, f"{api_url_base}/v0")
    t3_stop = perf_counter()

    # Delete all the files created by test
    delete_folder(offline_tx_directory)

    # Display test stats per test
    print_stats(no_of_txs_per_source, api_url_base_list, dst_addr_list, test_start_date, t1_start, t1_stop, t2_start,
                t2_stop, t3_stop)


if __name__ == '__main__':
    # generate_txs(no_of_sources, no_of_txs_per_source, api_url_base_list, faucet_addr_sk):
    generate_txs(10, 200, ["http://127.0.0.1:9001/api", "http://127.0.0.1:9002/api"],
                 "ed25519e_sk18r7nd20gaxjfgmahyqu2vngv98leqefcdcft2nevcakpf999spx55t4ph8ryqslp6ac7uryekjcqsqzl63rjpmh0k92dvquesweq38cc8a0wc")
