#!/usr/bin/env bash

diff <(cat insertions.txt queries.txt | cargo run --example blockindex | sort) <(cat insertions.txt | awk '{print $3}' | sort)

if [ $? -eq 0 ]
then
  echo "Success"
else
  echo ":(" >&2
fi
