import json


ELAPSED_TIME_ID = {'Other': 'request_elapsed_time'}


def get_data(line: str):
    try:
        return json.loads(line.split(" ", 2)[-1])
    except json.JSONDecodeError:
        return None


if __name__ == "__main__":
    import sys
    data_stream = iter(sys.stdin)
    jsons = (x for x in map(get_data, data_stream) if x)
    elapsed_time_logs = [x for x in jsons if x["id"] == ELAPSED_TIME_ID]
    average_request_time = sum(
        int(x["metadata"]["elapsed_nano_seconds"]) for x in elapsed_time_logs
    )/len(elapsed_time_logs)
    print(f"Average request time: {average_request_time}ns")
