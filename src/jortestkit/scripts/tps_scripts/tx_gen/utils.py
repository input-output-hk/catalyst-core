import json
import os
import re
import shutil
import subprocess
import time
import requests
from requests import HTTPError
import pandas as pd


def endpoint(url):
    try:
        r = requests.get(url)
        r.raise_for_status()
    except HTTPError as http_err:
        print("\nWeb API unavailable.\nError Details:\n")
        print(f"HTTP error occurred: {http_err}")
        exit(1)
    except Exception as err:
        print("\nWeb API unavailable.\nError Details:\n")
        print(f"Other error occurred: {err}")
        exit(1)
    else:
        return r


def get_api(api_url, path):
    r = endpoint(f'{api_url}/{path}')
    return r.text


def get_tip(api_url):
    return get_api(api_url, "tip")


def stats(api_url):
    r = endpoint(f'{api_url}/node/stats')
    print('Current node stats:\n')
    print(json.dumps(r.json(), sort_keys=True, indent=2))


def network_stats(api_url):
    r = endpoint(f'{api_url}/network/stats')
    print('Current network stats:\n')
    print(json.dumps(r.json(), sort_keys=True, indent=2))


def settings(api_url):
    r = endpoint(f'{api_url}/settings')
    # print('Current blockchain settings:\n')
    return json.dumps(r.json(), sort_keys=True, indent=2)


def stake(api_url):
    r = endpoint(f'{api_url}/stake')
    print('Current stake:\n')
    print(json.dumps(r.json(), sort_keys=True, indent=2))


def stake_pools(api_url):
    r = endpoint(f'{api_url}/stake_pools')
    print('Current stake pools:\n')
    print(json.dumps(r.json(), sort_keys=True, indent=2))


def utxo(api_url):
    r = endpoint(f'{api_url}/utxo')
    print('Current utxo:\n')
    print(json.dumps(r.json(), sort_keys=True, indent=2))


# def account(acc_addr):
# need to use the hex value of the acc_addr
#     r = endpoint(f'{api_url}/block/{acc_addr}')
#     print('Account details:\n')
#     print(json.dumps(r.json(), sort_keys=True, indent=2))


def get_block(api_url, block_id):
    r = endpoint(f'{api_url}/block/{block_id}')
    hex_block = r.content.hex()
    return hex_block


def parse_block(block):
    return {
        "epoch": int(block[16:24], 16),
        "slot": int(block[24:32], 16),
        "parent": block[104:168],
        "pool": block[168:232],
    }


def get_current_date(api_url):
    block_hash = get_tip(api_url)
    block = parse_block(get_block(api_url, block_hash))
    current_epoch = block['epoch']
    current_slot = block['slot']
    return str(current_epoch) + "." + str(current_slot)


def wait_new_block_created(timeout_no_of_blocks, api_url):
    print("Waiting for a new block to be created")
    counter = timeout_no_of_blocks
    initial_tip = get_tip(api_url)
    actual_tip = get_tip(api_url)

    while actual_tip == initial_tip:
        time.sleep(slot_duration)
        actual_tip = get_tip(api_url)
        counter = counter - 1
        if counter < 1:
            print("!!! ERROR: No block was created in last " + str(timeout_no_of_blocks * slot_duration) + " seconds")
            exit(1)
    print("New block was created - " + actual_tip)


def get_message_logs(api_url_base):
    try:
        cmd = "jcli rest v0 message logs --host " + api_url_base + " --output-format json"
        return json.loads(subprocess.check_output(cmd, shell=True, stderr=subprocess.STDOUT).decode("utf-8").strip())
    except subprocess.CalledProcessError as e:
        raise RuntimeError("command '{}' return with error (code {}): {}".format(e.cmd, e.returncode, e.output))


def get_message_logs_status_summary(api_url_base):
    status_counts = dict()
    message_logs = get_message_logs(api_url_base)
    for log in message_logs:
        if str(list(log["status"])[0]) not in list(status_counts):
            status_counts[str(list(log["status"])[0])] = 1
        else:
            status_counts[str(list(log["status"])[0])] += 1
    return status_counts


def read_blockchain_parameters(api_url):
    global block_0_hash, max_txs_per_block, slot_duration, slots_per_epoch, fee_constant, fee_coefficient, fee_certificate
    params = []
    blockchain_settings = settings(api_url)
    try:
        block_0_hash = re.search('"block0Hash": "(.+?)",', blockchain_settings).group(1)
    except AttributeError:
        print(
            "!!! ERROR: 'block0Hash' parameter was not found inside the node settings end point response;\n" + blockchain_settings)
    try:
        max_txs_per_block = int(re.search('"maxTxsPerBlock": (.+?),', blockchain_settings).group(1))
    except AttributeError:
        print(
            "!!! ERROR: 'maxTxsPerBlock' parameter was not found inside the node settings end point response;\n" + blockchain_settings)
    try:
        slot_duration = int(re.search('"slotDuration": (.+?),', blockchain_settings).group(1))
    except AttributeError:
        print(
            "!!! ERROR: 'slotDuration' parameter was not found inside the node settings end point response;\n" + blockchain_settings)
    try:
        slots_per_epoch = int(re.search('"slotsPerEpoch": (.+?)\s|$', blockchain_settings).group(1))
    except AttributeError:
        print(
            "!!! ERROR: 'slotsPerEpoch' parameter was not found inside the node settings end point response;\n" + blockchain_settings)
    try:
        fee_constant = int(re.search('"constant": (.+?)\s|$', blockchain_settings).group(1))
    except AttributeError:
        print(
            "!!! ERROR: 'fee_constant' parameter was not found inside the node settings end point response;\n" + blockchain_settings)
    try:
        fee_coefficient = int(re.search('"coefficient": (.+?),', blockchain_settings).group(1))
    except AttributeError:
        print(
            "!!! ERROR: 'fee_coefficient' parameter was not found inside the node settings end point response;\n" + blockchain_settings)
    try:
        fee_certificate = int(re.search('"certificate": (.+?),', blockchain_settings).group(1))
    except AttributeError:
        print(
            "!!! ERROR: 'fee_certificate' parameter was not found inside the node settings end point response;\n" + blockchain_settings)
    params.append(block_0_hash)
    params.append(max_txs_per_block)
    params.append(slot_duration)
    params.append(slots_per_epoch)
    params.append(fee_constant)
    params.append(fee_coefficient)
    params.append(fee_certificate)
    return params


def get_logs_summary(api_url_base):
    message_logs_json = get_message_logs(api_url_base)
    df = pd.DataFrame(message_logs_json)
    df['node_port'] = api_url_base

    print(f"================= Node {api_url_base} - Fragment Status Counts ====================")
    if len(df.columns) < 2:
        print(f"=== No message logs for node {api_url_base}")
    else:
        df['status'] = df['status'].astype(str)
        mined_txs_df = df[df['status'].str.contains("InABlock", na=False)][['fragment_id', 'status']]
        pending_txs_df = df[df['status'].str.contains("Pending", na=False)][['fragment_id', 'status']]
        rejected_txs_df = df[df['status'].str.contains("Rejected", na=False)][['fragment_id', 'status']]

        print(f"InABlock: {len(mined_txs_df)} fragments")
        print(f"Pending : {len(pending_txs_df)} fragments")
        print(f"Rejected: {len(rejected_txs_df)} fragments")

        print(f"======= Node {api_url_base} - Rejected reason count =======")
        print(rejected_txs_df['status'].value_counts())

        print(f"======= Node {api_url_base} - Rejected fragment_ids =======")
        print(rejected_txs_df['fragment_id'].values)

        # print(f"======= Node {node_port} - Pending fragment_ids =======")
        # print(pending_txs_df['fragment_id'].values)

        print(f"======= Node {api_url_base} - Duplicated Fragment IDs ======= ")
        duplicated_data = [g for _, g in df.groupby('fragment_id') if len(g) > 1]
        if len(duplicated_data) > 0:
            print(pd.concat(duplicated_data)[['fragment_id', 'status']])

        print(f"================= Node {api_url_base} - Transactions per block ====================")
        print(df['status'].value_counts().head(10))


def get_official_history(no_of_epochs, api_url):
    block_hash = get_tip(api_url)
    block = parse_block(get_block(api_url, block_hash))
    current_epoch = block['epoch']
    row_list = []

    while block['epoch'] > current_epoch - no_of_epochs:
        row_dict = {'date': str(block['epoch']) + '.' + str(block['slot']),
                    'block_hash': block_hash,
                    'parent_hash': block['parent'],
                    'pool_id': block['pool']}
        row_list.append(row_dict)

        block_hash = block['parent']
        block = parse_block(get_block(api_url, block['parent']))

    return pd.DataFrame(row_list, columns=['date', 'block_hash', 'parent_hash', 'pool_id'])


def delete_files(location_offline_tx_files):
    print(f"=================== Removing all the files from the {location_offline_tx_files} folder...")
    for fname in os.listdir(location_offline_tx_files):
        os.remove(os.path.join(location_offline_tx_files, fname))


def delete_folder(location_offline_tx_folder):
    print(f"=================== Removing the {location_offline_tx_folder} folder...")
    if os.path.exists(location_offline_tx_folder) and os.path.isdir(location_offline_tx_folder):
        try:
            shutil.rmtree(location_offline_tx_folder)
        except OSError as e:
            print("!!! Error: %s - %s." % (e.filename, e.strerror))
    else:
        print(f"Folder does not exists - {location_offline_tx_folder}")


def sorted_nicely(strings):
    # Sort strings the way humans are said to expect
    return sorted(strings, key=natural_sort_key)


def natural_sort_key(key):
    return [int(t) if t.isdigit() else t for t in re.split(r'(\d+)', key)]
