import argparse
import json
import subprocess

import pandas as pd

# USAGE:
#   python logs_analyzer.py -l 9001 9002 9003
#         - displays a summary of the status counts for each node (InABlock, Pending, Rejected)
#         - for Rejected fragments, it will display a summary with the Reasons, per node
#         - merges the message logs of all the nodes and returns the duplicated fragments
#           (fragments with different values on different nodes)
#   python logs_analyzer.py -l 9001 -t
#         - display first 10 epochs with the highest no of mined txs based on the message logs, per node
#   python logs_analyzer.py -l 9001 -f e1519a849a74fc42d4db52fffcdbb750e8cd19707a94bc99ba195b31ad7223ac
#         - returns the specific fragment_id entry from the message logs, per node (node_port)
#   python logs_analyzer.py -l 9001 -b d08aadd832f94dbf014830f954e35cca0eae326d464a86ee3080ca0ed2c889ed
#         - returns for the specific block inside the leader logs, per node (node_port)
#   python logs_analyzer.py -l 9001 9002 -d
#         - concat the leader logs and returns the entries with the same date(epoch.slot) and different block values

pd.set_option('display.max_columns', None)  # or 1000
pd.set_option('display.max_rows', None)  # or 1000
pd.set_option('display.max_colwidth', -1)  # or 199


def get_message_logs(rest_listen_port):
    api_url_base = f"http://127.0.0.1:{rest_listen_port}/api"
    try:
        cmd = "jcli rest v0 message logs --host " + api_url_base + " --output-format json"
        return json.loads(subprocess.check_output(cmd, shell=True, stderr=subprocess.STDOUT).decode("utf-8").strip())
    except subprocess.CalledProcessError as e:
        raise RuntimeError("command '{}' return with error (code {}): {}".format(e.cmd, e.returncode, e.output))


def get_leader_logs(rest_listen_port):
    api_url_base = f"http://127.0.0.1:{rest_listen_port}/api"
    try:
        cmd = "jcli rest v0 leaders logs get --host " + api_url_base + " --output-format json"
        return json.loads(subprocess.check_output(cmd, shell=True, stderr=subprocess.STDOUT).decode("utf-8").strip())
    except subprocess.CalledProcessError as e:
        raise RuntimeError("command '{}' return with error (code {}): {}".format(e.cmd, e.returncode, e.output))


def main():
    if vars(args)["duplicated_date"]:
        concat_df_leader = pd.DataFrame()
        for node_port in vars(args)["list_node_ports"]:
            leader_logs_json = get_leader_logs(node_port)
            df_leader = pd.DataFrame(leader_logs_json)
            df_leader['node_port'] = node_port
            df_leader['status'] = df_leader['status'].astype(str)
            df_leader = df_leader.drop(['scheduled_at_time', 'created_at_time', 'enclave_leader_id', 'finished_at_time', 'wake_at_time'], axis=1)
            concat_df_leader = pd.concat([concat_df_leader, df_leader]).drop_duplicates(
                subset=['scheduled_at_date', 'status']).reset_index(drop=True)
        if 'scheduled_at_date' in concat_df_leader.columns and 'status' in concat_df_leader.columns:
            dup_data_concat = [g for _, g in concat_df_leader.groupby('scheduled_at_date') if len(g) > 1]
            print(f"=== There are {len(dup_data_concat)} entries into the concat_df_leader --> check 'dup_dates_concat.txt' file for details")
            if len(dup_data_concat) > 0:
                with open('dup_dates_concat.txt', mode='wt', encoding='utf-8') as f:
                    for element in dup_data_concat:
                        f.write(str(element))
                        f.write('\n')
                        f.write('-----------------------------------------------------------------')
                        f.write('\n')
                    f.close()
        else:
            print("=== There are no logs into the concat_df")
    elif vars(args)["block_hash"]:
        for node_port in vars(args)["list_node_ports"]:
            leader_logs_json = get_leader_logs(node_port)
            df_leader = pd.DataFrame(leader_logs_json)
            df_leader['node_port'] = node_port
            df_leader['status'] = df_leader['status'].astype(str)
            print(f"================= Node {node_port} - Searching for block: {vars(args)['block_hash']} =============")
            print(df_leader[df_leader['status'].str.contains(vars(args)["block_hash"], na=False)][
                      ["scheduled_at_date", "status", "node_port"]])
    elif vars(args)["txs_per_epoch"]:
        for node_port in vars(args)["list_node_ports"]:
            message_logs_json = get_message_logs(node_port)
            df = pd.DataFrame(message_logs_json)
            if len(df.columns) < 2:
                print(f"=== No message logs for node {node_port}")
            else:
                print(f"================= Node {node_port} - Transactions per block/epoch ====================")
                print(df['status'].value_counts().head(10))
    elif vars(args)["fragment_id"]:
        for node_port in vars(args)["list_node_ports"]:
            message_logs_json = get_message_logs(node_port)
            df = pd.DataFrame(message_logs_json)
            df['node_port'] = node_port
            print(df.loc[df['fragment_id'] == vars(args)["fragment_id"]][["status", "node_port"]])
    else:
        concat_df = pd.DataFrame()
        for node_port in vars(args)["list_node_ports"]:
            message_logs_json = get_message_logs(node_port)
            df = pd.DataFrame(message_logs_json)
            df['node_port'] = node_port

            print(f"================= Node {node_port} - Fragment Status Counts ====================")
            if len(df.columns) < 2:
                print(f"=== No message logs for node {node_port}")
            else:
                df['status'] = df['status'].astype(str)
                mined_txs_df = df[df['status'].str.contains("InABlock", na=False)][['fragment_id', 'status']]
                pending_txs_df = df[df['status'].str.contains("Pending", na=False)][['fragment_id', 'status']]
                rejected_txs_df = df[df['status'].str.contains("Rejected", na=False)][['fragment_id', 'status']]

                print(f"InABlock: {len(mined_txs_df)} fragments")
                print(f"Pending : {len(pending_txs_df)} fragments")
                print(f"Rejected: {len(rejected_txs_df)} fragments")

                print(f"======= Node {node_port} - Rejected reason count =======")
                print(rejected_txs_df['status'].value_counts())

                print(f"======= Node {node_port} - Rejected fragment_ids =======")
                print(rejected_txs_df['fragment_id'])

                # print(f"======= Node {node_port} - Pending fragment_ids =======")
                # print(pending_txs_df['fragment_id'])

                print(f"======= Node {node_port} - Duplicated Fragment IDs ======= ")
                duplicated_data = [g for _, g in df.groupby('fragment_id') if len(g) > 1]
                if len(duplicated_data) > 0:
                    print(pd.concat(duplicated_data)[['fragment_id', 'status']])

                concat_df = pd.concat([concat_df, df]).drop_duplicates(subset=['fragment_id', 'status']).reset_index(
                    drop=True)

        print("#######################################################################################################")

        print("================= Compare the status of Pending and Rejected fragments on all nodes ===================")
        if 'fragment_id' in concat_df.columns and 'status' in concat_df.columns:
            dup_data_concat = [g for _, g in concat_df.groupby('fragment_id') if len(g) > 1]
            print(
                f"=== There are {len(dup_data_concat)} into the concat_df --> check 'dup_data_concat.txt' file for details")
            if len(dup_data_concat) > 0:
                with open('dup_data_concat.txt', 'w+') as f:
                    print(pd.concat(dup_data_concat)[['fragment_id', 'status', 'node_port']], file=f)
        else:
            print("=== There are no logs into the concat_df")
    exit(0)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("-l", "--list_node_ports", nargs='+',
                        help="displays status summary for fragments (found on message logs)", required=True)
    parser.add_argument("-f", "--fragment_id", help="fragment id to be searched into message logs")
    parser.add_argument("-b", "--block_hash", help="block hash to be searched into leader logs")
    parser.add_argument("-d", "--duplicated_date", action="store_true",
                        help="display dates with multiple blocks inside the leader logs")
    parser.add_argument("-t", "--txs_per_epoch", action="store_true",
                        help="display no of txs per epochs (found on message logs)")

    args = parser.parse_args()
    main()
