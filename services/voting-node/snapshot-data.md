# Voter registrations and voting power


1. Fetch voting groups (currently: `"direct"`, and `"rep"`)
2. Set a voting token for each group for this event.
3. Fetch registered voters:
    1. Encode wallet key to Bech32
    2. Create `fund` fragment
    2. Create `token` fragment
