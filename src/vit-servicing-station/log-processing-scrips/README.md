# Log processing scripts


The set of scripts living in this folders are built to example how the logs from the `logging-lib` can be aggregated/processed
to recall interesting data/usage from the service.


- `average_request_response_time.py` : Process a log file checking response times for requests and returns the average request time.
It should reads from the stdin so it can be fed piping the d. For example: `cat mylog.log | python average_request_response_time.py` 