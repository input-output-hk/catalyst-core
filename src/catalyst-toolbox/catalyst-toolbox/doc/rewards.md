# Rewards data pipeline

The rewards process is an entangled system of data requirements which will 
be listed in the former document.


## Voters rewards

### Input

Currently, (as per Fund7) the tool needs:

* The block0 file (bin)
* The amount of rewards to distribute
* The threshold of votes a voter need in order to access such rewards

### Output 

A Csv is generated with the following headers:


```
+---------+---------------------------+----------------------------+-------------------------------+
| Address | Stake of the voter (ADA)  | Reward for the voter (ADA) |Reward for the voter (lovelace)|
+---------+---------------------------+----------------------------+-------------------------------+
```

## Proposers reward

Users that propose proposals get a rewards too. We can use the [`proposers_rewards.py`](https://github.com/input-output-hk/catalyst-toolbox#calculate-proposers-rewards) script for that.
The scrip has two modes of operating, online and offline. 
The online mode works with the data living in the vit-servicing-station server.
The offline mode need to load that data manually through some json files. 
Those json files can be downloaded from the vit-servicing-station at any time during the fund.

### Input

#### Json files needed
1. challenges: from `https://servicing-station.vit.iohk.io/api/v0/challenges`
2. active voteplans: from `https://servicing-station.vit.iohk.io/api/v0/vote/active/plans`
3. proposals: from `https://servicing-station.vit.iohk.io/api/v0/proposals`
4. excluded proposals: a json file with a list of excluded proposals ids `[id, ..idx]`

### Output

The proposers output is csv with several data on it. 
***Really important***, this output file is used as source of truth for the approved proposals 
(not to be mistaken with funded proposals).

Output csv headers:
* internal_id: proposal internal id (from vss)
* proposal_id: proposal chain id
* proposal: proposal title
* overall_score: proposal impact score
* yes: how many yes votes
* no: how many no votes
* result: yes vs no votes difference
* meets_approval_threshold: **is proposal approved**
* requested_dollars: amount of funding requested
* status: **is proposal funded**
* fund_depletion: fund remaining after proposal depletion (entries are sorted in descending order of 'result')
* not_funded_reason: why wasnt the proposal not funded (if applies, over budget or approval threshold)
* link_to_ideascale: url to ideascale proposal page

The output files are generated per challenge. So, if we have 30 challenges we would have 30 generated output files 
in the same fashion.


## Community advisors rewards

### Input

There are 2 (two) main input files needed for calculating the community advisors rewards:

1. Proposers reward result output file (approved proposals): We need this to check which of the proposals were approved. 
Notice that the proposers rewards script output is per challenge. So in order to use it we need to aggregate all the csv
into a single file (same headers, order is irrelevant). For this we can use the 
[`csv_merger.py`](https://github.com/input-output-hk/catalyst-toolbox/blob/main/catalyst-toolbox/scripts/python/csv_merger.py) script,
or any other handier tool.
2.Assessments csv (assessments): This is a file that comes from the community. It holds the information with the reviews performed
by the CAs.

### Output

 A csv with pairs of anonymize CA ids and the amount of the reward:

```
+----+----------+
| id | rewards  |
+----+----------+
```

## Veteran community advisors rewards

### Input

Currently, (as per fund7) it is just a normal distribution based on `(number_of_reviews/total_reviews)*total_rewards`

For that we just need to know the amount of rewards done by each veteran:

1. Veteran reviews count: A csv with pairs of `veteran_id -> total_reviews`. It is also a community based document 
(it is provided every fund). 


### Output

A csv with pairs of anonymize veteran CA ids and the amount of the reward, `veteran_id -> total_rewards`.
