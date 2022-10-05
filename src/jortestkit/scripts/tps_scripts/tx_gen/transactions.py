#!/usr/bin/env python3
import os
import re
import subprocess

from tx_gen.addresses import create_addr_from_sk_key
from tx_gen.utils import wait_new_block_created

ADDRTYPE = "--testing"


def create_offline_tx_file(tx_file):
    try:
        cmd = "jcli transaction new --staging " + tx_file
        return subprocess.check_output(cmd, shell=True, stderr=subprocess.STDOUT).decode("utf-8").strip()
    except subprocess.CalledProcessError as e:
        raise RuntimeError("command '{}' return with error (code {}): {}".format(e.cmd, e.returncode, e.output))


def add_input_account(tx_file, src_acc_addr, src_amount_fee_included):
    try:
        cmd = "jcli transaction add-account " + src_acc_addr + " " + str(src_amount_fee_included) + " --staging " + tx_file
        return subprocess.check_output(cmd, shell=True, stderr=subprocess.STDOUT).decode("utf-8").strip()
    except subprocess.CalledProcessError as e:
        raise RuntimeError("command '{}' return with error (code {}): {}".format(e.cmd, e.returncode, e.output))


def add_output_account(tx_file, dst_addr, dst_amount):
    try:
        cmd = "jcli transaction add-output " + dst_addr + " " + str(dst_amount) + " --staging " + tx_file
        return subprocess.check_output(cmd, shell=True, stderr=subprocess.STDOUT).decode("utf-8").strip()
    except subprocess.CalledProcessError as e:
        raise RuntimeError("command '{}' return with error (code {}): {}".format(e.cmd, e.returncode, e.output))


def finalize_tx(tx_file):
    try:
        cmd = "jcli transaction finalize --staging " + tx_file
        return subprocess.check_output(cmd, shell=True, stderr=subprocess.STDOUT).decode("utf-8").strip()
    except subprocess.CalledProcessError as e:
        raise RuntimeError("command '{}' return with error (code {}): {}".format(e.cmd, e.returncode, e.output))


def get_tx_data(tx_file):
    try:
        cmd = "jcli transaction data-for-witness --staging " + tx_file
        return subprocess.check_output(cmd, shell=True, stderr=subprocess.STDOUT).decode("utf-8").strip()
    except subprocess.CalledProcessError as e:
        raise RuntimeError("command '{}' return with error (code {}): {}".format(e.cmd, e.returncode, e.output))


def make_witness(tx_data, addr_type, account_spending_counter, block_0_hash, witness_out_file, witness_secret_file):
    try:
        cmd = "jcli transaction make-witness " + tx_data + " --genesis-block-hash " + block_0_hash + " --type " + \
              addr_type + " --account-spending-counter " + str(account_spending_counter) + " " + witness_out_file \
              + " " + witness_secret_file
        return subprocess.check_output(cmd, shell=True, stderr=subprocess.STDOUT).decode("utf-8").strip()
    except subprocess.CalledProcessError as e:
        raise RuntimeError("command '{}' return with error (code {}): {}".format(e.cmd, e.returncode, e.output))


def add_witness(tx_file, witness_out_file):
    try:
        cmd = "jcli transaction add-witness " + witness_out_file + " --staging " + tx_file
        return subprocess.call(cmd, shell=True, stderr=subprocess.STDOUT)
    except subprocess.CalledProcessError as e:
        raise RuntimeError("command '{}' return with error (code {}): {}".format(e.cmd, e.returncode, e.output))


def get_tx_info(tx_file, fee_constant, fee_coefficient, fee_certificate):
    try:
        cmd = "jcli transaction info " + "--fee-constant " + str(fee_constant) + " --fee-coefficient " + \
              str(fee_coefficient) + " --fee-certificate " + str(fee_certificate) + " --staging " + tx_file
        return subprocess.check_output(cmd, shell=True, stderr=subprocess.STDOUT).decode("utf-8").strip()
    except subprocess.CalledProcessError as e:
        raise RuntimeError("command '{}' return with error (code {}): {}".format(e.cmd, e.returncode, e.output))


def seal_tx(tx_file):
    try:
        cmd = "jcli transaction seal --staging " + tx_file
        return subprocess.check_output(cmd, shell=True, stderr=subprocess.STDOUT).decode("utf-8").strip()
    except subprocess.CalledProcessError as e:
        raise RuntimeError("command '{}' return with error (code {}): {}".format(e.cmd, e.returncode, e.output))


def tx_to_message(tx_file):
    # place where the transaction is going to be save during its staging phase If a file
    # is given, the transaction will be read from this file and modification will be
    # written into this same file.
    try:
        cmd = "jcli transaction to-message --staging " + tx_file
        return subprocess.check_output(cmd, shell=True, stderr=subprocess.STDOUT).decode("utf-8").strip()
    except subprocess.CalledProcessError as e:
        raise RuntimeError("command '{}' return with error (code {}): {}".format(e.cmd, e.returncode, e.output))


def get_account_details(account_addr, api_url_base):
    try:
        cmd = "jcli rest v0 account get " + account_addr + " --host  " + api_url_base
        ps = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        result = ps.communicate()[0].decode("utf-8").strip()

        if 'failed to make a REST request' in result:
            print("!!! ERROR: Client Error: 404 Not Found - Account is not yet known by the blockchain")
        else:
            spending_counter = int(re.search('counter: (.+?)\s|$', result).group(1))
            value = int(re.search('value: (.+?)$', result).group(1))
            return spending_counter, value
    except subprocess.CalledProcessError as e:
        raise RuntimeError("command '{}' return with error (code {}): {}".format(e.cmd, e.returncode, e.output))


def post_tx(offline_tx_file, api_url_base):
    # posts a signed, hex-encoded transaction; Fragment Id is printed on success
    try:
        cmd = "jcli rest v0 message post --file " + offline_tx_file + " --host " + api_url_base
        return subprocess.call(cmd, shell=True, stderr=subprocess.STDOUT)
    except subprocess.CalledProcessError as e:
        raise RuntimeError("command '{}' return with error (code {}): {}".format(e.cmd, e.returncode, e.output))


def encode_and_post_tx(offline_tx_file, api_url_base):
    # encodes and posts a signed, hex-encoded transaction; Fragment Id is printed on success
    try:
        cmd = "jcli transaction to-message --staging " + offline_tx_file + \
              ' | jcli rest v0 message post --host ' + api_url_base
        ps = subprocess.Popen(cmd, shell=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        return ps.communicate()[0].decode("utf-8").strip()
    except subprocess.CalledProcessError as e:
        raise RuntimeError("command '{}' return with error (code {}): {}".format(e.cmd, e.returncode, e.output.decode("utf-8")))


def encode_and_post_txs(offline_txs_list, api_url_base, location_offline_tx_files):
    print(f"=================== Encoding and posting {len(offline_txs_list)} transactions =====================")
    for tx in offline_txs_list:
        encode_and_post_tx(location_offline_tx_files + tx, api_url_base)
        # tx_hash = encode_and_post_tx(location_offline_tx_files + tx, api_url_base)


def create_tx_send_funds_acc_to_acc(offline_tx_name, dst_addr, dst_amount, src_add_sk, fee_constant, fee_coefficient,
                                    fee_certificate, block_0_hash, witness_out_file, witness_secret_file, src_spending_counter):
    create_offline_tx_file(offline_tx_name)
    src_addr = create_addr_from_sk_key(src_add_sk, "account")
    # TO DO: src_amount_fee_included is hardcoded to 1 Source and 1 Destination (make it generic if required)
    src_amount_fee_included = dst_amount + fee_constant + 2 * fee_coefficient
    add_input_account(offline_tx_name, src_addr, src_amount_fee_included)
    add_output_account(offline_tx_name, dst_addr, dst_amount)
    finalize_tx(offline_tx_name)
    tx_data = get_tx_data(offline_tx_name)
    make_witness(tx_data, "account", src_spending_counter, block_0_hash, witness_out_file, witness_secret_file)
    add_witness(offline_tx_name, witness_out_file)
    seal_tx(offline_tx_name)
    # tx_to_message(offline_tx_name)


def send_funds_from_acc_to_acc_list(offline_tx_directory, src_add_sk, dst_addr_list, tx_value, api_url_base,
                                    fee_constant, fee_coefficient, fee_certificate, block_0_hash):
    """
    This method can be used to send funds from 1 account to a list of accounts;
    It will send the same amount of funds to all the destination accounts - tx_value
    """
    src_addr = create_addr_from_sk_key(src_add_sk, "account")
    src_addr_balance = get_account_details(src_addr, api_url_base)[1]
    src_addr_counter = get_account_details(src_addr, api_url_base)[0]

    src_tx_value = tx_value + fee_constant + 2 * fee_coefficient

    print("=============================================================================")
    print(f"Number of destinations  : {len(dst_addr_list)}")
    print(f"Node details            : {api_url_base}")
    print(f"Source Address          : {src_addr}")
    print(f"Source Balance          : {src_addr_balance}")
    print(f"Source Counter          : {src_addr_counter}")
    print(f"Destination Addresses   : {dst_addr_list}")
    print(f"TX Value (per dst addr) : {tx_value}")
    print(f"Required source funds   : {src_tx_value * len(dst_addr_list)}")
    print("=============================================================================")

    # create the offline transactions
    location_for_offline_tx_files = offline_tx_directory + src_add_sk[-5:] + "/"
    os.makedirs(os.path.dirname(location_for_offline_tx_files), exist_ok=True)

    offline_txs_list = []
    tx_counter = 0
    for dst_acc_addr in dst_addr_list:
        offline_tx_name = "tx_" + str(tx_counter)
        offline_tx = location_for_offline_tx_files + offline_tx_name
        witness_out_file = location_for_offline_tx_files + "witness_tx_" + str(tx_counter)
        witness_secret_file = location_for_offline_tx_files + "witness_secret_tx_" + str(tx_counter)
        offline_txs_list.append(offline_tx_name)

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
        # print(get_tx_info(offline_tx, fee_constant, fee_coefficient, fee_certificate))

        # remove the witness files (not needed after the transaction was created)
        for fname in os.listdir(location_for_offline_tx_files):
            if fname.startswith("witness"):
                os.remove(os.path.join(location_for_offline_tx_files, fname))
        tx_counter += 1

    encode_and_post_txs(offline_txs_list, api_url_base, location_for_offline_tx_files)

    # Wait for 1 new block to be created - as part of the measurement duration definition
    wait_new_block_created(200, f"{api_url_base}/v0")
