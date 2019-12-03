#!/usr/bin/env python3

import subprocess

ADDRTYPE = "--testing"
addr_details = {}


def create_private_key():
    try:
        cmd = "jcli key generate --type=ed25519extended"
        return subprocess.check_output(cmd, shell=True, stderr=subprocess.STDOUT).decode("utf-8").strip()
    except subprocess.CalledProcessError as e:
        raise RuntimeError("command '{}' return with error (code {}): {}".format(e.cmd, e.returncode, e.output))


def create_public_key(private_key):
    try:
        cmd = 'echo ' + private_key + ' | jcli key to-public'
        ps = subprocess.Popen(cmd,shell=True,stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        return ps.communicate()[0].decode("utf-8").strip()
    except subprocess.CalledProcessError as e:
        raise RuntimeError("command '{}' return with error (code {}): {}".format(e.cmd, e.returncode, e.output.decode("utf-8")))


def create_account_addr(public_key):
    try:
        cmd = "jcli address account " + public_key + " " + ADDRTYPE
        return subprocess.check_output(cmd, shell=True, stderr=subprocess.STDOUT).decode("utf-8").strip()
    except subprocess.CalledProcessError as e:
        raise RuntimeError("command '{}' return with error (code {}): {}".format(e.cmd, e.returncode, e.output))


def create_utxo_addr(public_key):
    try:
        cmd = "jcli address single " + public_key + " " + ADDRTYPE
        subprocess.check_output(cmd, shell=True, stderr=subprocess.STDOUT).decode("utf-8").strip()
    except subprocess.CalledProcessError as e:
        raise RuntimeError("command '{}' return with error (code {}): {}".format(e.cmd, e.returncode, e.output))


def create_addr_from_sk_key(private_key, addr_type):
    public_key = create_public_key(private_key)
    if addr_type == "account":
        return create_account_addr(public_key)
    elif addr_type == "utxo":
        return create_utxo_addr(public_key)
    else:
        print("!!! ERROR: valid arguments are: utxo, account")
        return None


def create_address(addr_type):
    private_key = create_private_key()
    addr_details["sk"] = private_key
    public_key = create_public_key(private_key)
    addr_details["pk"] = public_key
    if addr_type == "account":
        acc_addr = create_account_addr(public_key)
        addr_details["addr"] = acc_addr
        return addr_details
    elif addr_type == "utxo":
        utxo_addr = create_utxo_addr(public_key)
        addr_details["addr"] = utxo_addr
        return addr_details
    else:
        print("!!! ERROR: valid arguments are: utxo, account")
