### Community Tally verification tool

This tool allows the community to cross-reference and validate vote results.

*Example usage:*

```
cargo build --release -p audit
```  

#### The fragment analysis tool saves the ledgers before and after state.
#### The before state contains the encrypted tallies and the after state contains the decrypted tallies.

I want to verify a proposal:

- Open *ledger_after_tally.json* and find the proposal id of the the result in question.

We need three ingredients to unwind the cryptographic proof.
- [ ] Encrypted tally
- [ ] Public keys of committee
- [ ] Decryption shares

### Encrypted tally
- Open *ledger_before_tally.json* and find the encrypted tally for the proposal id.
```bash
ENCRYPTED_TALLY='yF6iIwbAVAsIBryA+HDWlL8X4G8TCrP1G47O8ossgT3jwrQFAAAAAOzRYqMSRlR0RHwFKLcH+OStqQfWeRLlCPyXvsAjNV0ixJ6FrijYPUnKt8bG6xcQ5ROQmzvTc2MrC47VAYNIU1Ls0WKjEkZUdER8BSi3B/jkrakH1nkS5Qj8l77AIzVdIlQanikT1SrvHlKmDn+9E1stq3uxmlFqm2jSgCVzcKN/'
```

### Committee Public keys

- Search for *committee_member_keys* in *ledger_before_tally.json* - use the public key associated with the vote plan id where the given proposal id resides.Each vote plan will have a unique member public key. Make sure the member public key coincides with proposal id in question.

```bash
PUBLIC_KEY_ALICE='ristretto255_memberpk1mrt3dr5vher3jhs7enepaxl65ggsqeu23amm55nxup9h8nqmuqeqwccn78' 
```
### Decryption shares
- Open *decryption_shares.json* and find the decryption shares for the proposal id by searching for the plaintext results of the proposal id.
```bash
SHARES_ALICE='WDDMb68A6JCVR5UdhDtl7QYrQHSMOFqg44lHcmtB/Q3IfSoqusq+obtC/JJOtDYWadSM9mOXtPwUfwV14hrGAw30MilDYi93ULxgB9JZ8+hlTaCkH4Dr3y3zALLBS6UEWDDMb68A6JCVR5UdhDtl7QYrQHSMOFqg44lHcmtB/Q2Q9hcGlcFVV4QLXxlWOAb1hJT9/2WhM16JXyJ+RC3MAyUKJf2AJJGuENKWyPEROI7ROuiVU6hn/iVIVbGQeO0I'
```

#### Public use: Validate results 
#### `decrypt_tally_from_shares(pub_keys, encrypted_tally, decrypt_shares) -> tallyResultPlaintext`

```bash
SHARES_ALICE='0DDHBs4TnabGQjvhIiQP2S53mTThxqilR+ogY8MpIRV6PdDb+5NWVZYQvAQQZUIe8e/rzeZjGX5QkCpd84b/CyvrivhD4u7zvhccz6zSgOfLx3EVjY9PXBXOhPYkrUoE0DDHBs4TnabGQjvhIiQP2S53mTThxqilR+ogY8MpIRXOp9W5weElw0uZSyz4oCkRMKiRv2L1kfrOuNLOXtobBnWorfj2FLdBb2jZ5Cb0tqYvMKj+WLTTs2hrohjlSC0D'

ENCRYPTED_TALLY='yF6iIwbAVAsIBryA+HDWlL8X4G8TCrP1G47O8ossgT3jwrQFAAAAAOzRYqMSRlR0RHwFKLcH+OStqQfWeRLlCPyXvsAjNV0ixJ6FrijYPUnKt8bG6xcQ5ROQmzvTc2MrC47VAYNIU1Ls0WKjEkZUdER8BSi3B/jkrakH1nkS5Qj8l77AIzVdIlQanikT1SrvHlKmDn+9E1stq3uxmlFqm2jSgCVzcKN/'

PUBLIC_KEY_ALICE='ristretto255_memberpk1mrt3dr5vher3jhs7enepaxl65ggsqeu23amm55nxup9h8nqmuqeqwccn78' 

./target/release/tally --decrypt-tally-from-shares $SHARES_ALICE --encrypted-tally $ENCRYPTED_TALLY --public-keys $PUBLIC_KEY_ALICE
```
N number of committee members

```bash
SHARES_ALICE='WDDMb68A6JCVR5UdhDtl7QYrQHSMOFqg44lHcmtB/Q3IfSoqusq+obtC/JJOtDYWadSM9mOXtPwUfwV14hrGAw30MilDYi93ULxgB9JZ8+hlTaCkH4Dr3y3zALLBS6UEWDDMb68A6JCVR5UdhDtl7QYrQHSMOFqg44lHcmtB/Q2Q9hcGlcFVV4QLXxlWOAb1hJT9/2WhM16JXyJ+RC3MAyUKJf2AJJGuENKWyPEROI7ROuiVU6hn/iVIVbGQeO0I'

SHARES_BOB='uMo4REGZf+UTSRNlheK/mLDm4rXm7tT+n6cCotRnkUH52QmrMMhtTD2juMO+wRPqByv2nhtlxkln17B3evCmA6ZwFFsamQ7gdIT/Iaob25kKz96fXS0EFZdoq2r8m74BuMo4REGZf+UTSRNlheK/mLDm4rXm7tT+n6cCotRnkUHG1m8tlj8qtr6M3e3G1V8iKrckpeTdj9BbLLYTDPVAA/gGaearM6ltl9DfB5Ageg/Ngt3F9xKYPj1buPCctqsD'

SHARES_CHARLIE='xk//jFz+eWIJ9qLxN2WDpBok/Fb3C/v3oZiwZmsou0NiD1MAUKef+/pVDtlRRNeG8YXT4Ywz3Q2nY+jHSNhdDEV/+9rXMk2K7oAeMSXOuXcE0rS4Yoj4OV7BScPjnpAFxk//jFz+eWIJ9qLxN2WDpBok/Fb3C/v3oZiwZmsou0NXgdo/LQfRujYuSXY38IOegq8xC+WN+f5wOfrRqmgQDdxlt1B4PYSDqolKH33CjIN+IBjrgkwNcpBzQb5Gk5IO'

ENCRYPTED_TALLY='rs9sAB/n6vaQh5NMH+UunES87fdcpA3QDll/AV8p1x8IAAAAAAAAAMZEhl0tYixbZzHhAXWklbdJbvwiJvfYidowNZ1KzUs/NKi98HtPNN3gdl1T+ehhNhxLFQ/7fTJSVjAJycNWhkDGRIZdLWIsW2cx4QF1pJW3SW78Iib32InaMDWdSs1LP0pyBYKkVTpExb78GZrf/8csqWtNQNshoLoHsa827gdF'

PUBLIC_KEY_ALICE='ristretto255_memberpk1qnh2q0ldl7juflfgj3jplm3dt8szx0wthx5992h2pr4n4z4zu4lsxkk8pz'

PUBLIC_KEY_BOB='ristretto255_memberpk1spxj8cjraus3ceu0kr6ad9g5parv9htl8j0clst6sg4ruc8u3elsmngewt'

PUBLIC_KEY_CHARLIE='ristretto255_memberpk1asgwda6h6690jmtyv8vclq268n7t755cxh8x683hq8urap4grsmqktyhc6'

echo $ENCRYPTED_TALLY

./target/release/tally --decrypt-tally-from-shares $SHARES_ALICE $SHARES_BOB $SHARES_CHARLIE --encrypted-tally $ENCRYPTED_TALLY --public-keys $PUBLIC_KEY_ALICE $PUBLIC_KEY_BOB $PUBLIC_KEY_CHARLIE
```

#### Internal use: Generate decrypt shares for publication 
#### `produce_decrypt_shares(secret_keys, encrypted_tally)-> decryptShares`

```bash
ENCRYPTED_TALLY='rs9sAB/n6vaQh5NMH+UunES87fdcpA3QDll/AV8p1x8IAAAAAAAAAMZEhl0tYixbZzHhAXWklbdJbvwiJvfYidowNZ1KzUs/NKi98HtPNN3gdl1T+ehhNhxLFQ/7fTJSVjAJycNWhkDGRIZdLWIsW2cx4QF1pJW3SW78Iib32InaMDWdSs1LP0pyBYKkVTpExb78GZrf/8csqWtNQNshoLoHsa827gdF'

ALICE_SECRET_KEY='ristretto255_membersk1e6445v082djlnky70t38ac5c9f4xxldhkyqst97dcwsqthzvvcyqh3f78t'

BOB_SECRET_KEY='ristretto255_membersk1cen98tnz4h5ndpwfjrrcq964jk77awaguwxxmd97f8455rtpdc8qp6ptwe'

CHARLIE_SECRET_KEY='ristretto255_membersk1392k23gzgwv827hdfjg3g9es0depszcz4t3glvjjkv7sufuqkc9q0nzrns'

 ./target/release/tally --produce-decrypt-shares $ALICE_SECRET_KEY $BOB_SECRET_KEY $CHARLIE_SECRET_KEY --encrypted-tally $ENCRYPTED_TALLY
```

#### Internal use: Decrypt tally 
#### `decrypt_tally_from_keys(secret_keys, encrypted_tally)-> tallyResultPlaintext`

```bash 
ENCRYPTED_TALLY='rs9sAB/n6vaQh5NMH+UunES87fdcpA3QDll/AV8p1x8IAAAAAAAAAMZEhl0tYixbZzHhAXWklbdJbvwiJvfYidowNZ1KzUs/NKi98HtPNN3gdl1T+ehhNhxLFQ/7fTJSVjAJycNWhkDGRIZdLWIsW2cx4QF1pJW3SW78Iib32InaMDWdSs1LP0pyBYKkVTpExb78GZrf/8csqWtNQNshoLoHsa827gdF'

ALICE_SECRET_KEY='ristretto255_membersk1e6445v082djlnky70t38ac5c9f4xxldhkyqst97dcwsqthzvvcyqh3f78t'

BOB_SECRET_KEY='ristretto255_membersk1cen98tnz4h5ndpwfjrrcq964jk77awaguwxxmd97f8455rtpdc8qp6ptwe'

CHARLIE_SECRET_KEY='ristretto255_membersk1392k23gzgwv827hdfjg3g9es0depszcz4t3glvjjkv7sufuqkc9q0nzrns'

 ./target/release/tally --decrypt-tally-from-keys $ALICE_SECRET_KEY $BOB_SECRET_KEY $CHARLIE_SECRET_KEY --encrypted-tally $ENCRYPTED_TALLY
```

#### Internal use: Show public keys of private keys 
#### `show_public_keys(secret_keys)-> PubKeys`

```bash
ALICE_SECRET_KEY='ristretto255_membersk1e6445v082djlnky70t38ac5c9f4xxldhkyqst97dcwsqthzvvcyqh3f78t'

BOB_SECRET_KEY='ristretto255_membersk1cen98tnz4h5ndpwfjrrcq964jk77awaguwxxmd97f8455rtpdc8qp6ptwe'

CHARLIE_SECRET_KEY='ristretto255_membersk1392k23gzgwv827hdfjg3g9es0depszcz4t3glvjjkv7sufuqkc9q0nzrns'

 ./target/release/tally --show-public-keys $ALICE_SECRET_KEY $BOB_SECRET_KEY $CHARLIE_SECRET_KEY
```
