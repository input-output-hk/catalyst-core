--sql
-- Data from Catalyst Fund 4

-- Purge all Fund 4 data before re-inserting it.
DELETE FROM event WHERE row_id = 4;

-- Load the raw Block0 Binary from the file.
\set block0path 'historic_data/fund_4/block0.bin'
\set block0contents `base64 :block0path`

-- Create the Event record for Fund 3

INSERT INTO event
(row_id, name, description,
 start_time,
 end_time,
 registration_snapshot_time,
 snapshot_start,
 voting_power_threshold,
 max_voting_power_pct,
 insight_sharing_start,
 proposal_submission_start,
 refine_proposals_start,
 finalize_proposals_start,
 proposal_assessment_start,
 assessment_qa_start,
 voting_start,
 voting_end,
 tallying_end,
 block0,
 block0_hash,
 committee_size,
 committee_threshold)
VALUES

(4, 'Catalyst Fund 4', 'Create, fund and deliver the future of Cardano.',
 '2021-02-17 22:00:00', -- Start Time - Date/Time accurate.
 '2021-07-04 11:00:00', -- End Time   - Date/Time accurate.
 '2021-06-11 11:00:26', -- Registration Snapshot Time - Date/time Accurate. Slot 31842935
                        -- Vit-SS Says 2021-06-11 11:00:00
 '2021-06-12 11:00:00', -- Snapshot Start - Date/time Unknown.
 450000000,            -- Voting Power Threshold -- Accurate
 100,                   -- Max Voting Power PCT - No max% threshold used in this fund.
 NULL,                  -- Insight Sharing Start - None
 '2021-02-24 22:00:00', -- Proposal Submission Start - Date/time accurate.
 '2021-03-03 22:00:00', -- Refine Proposals Start - Date/time accurate.
 '2021-03-10 22:00:00', -- Finalize Proposals Start - Date/time accurate.
 '2021-03-17 19:00:00', -- Proposal Assessment Start - Date/time accurate.
 '2021-03-24 19:00:00', -- Assessment QA Start - Datetime accurate.
 '2021-06-15 07:23:06', -- Voting Starts - Date/time accurate.
 '2021-06-25 11:23:06', -- Voting Ends - Date/time Accurate.
 '2021-07-05 11:23:06', -- Tallying Ends - Date/time Accurate.
 decode(:'block0contents','base64'),
                        -- Block 0 Data - From File
 NULL,                  -- Block 0 Hash - TODO
 0,                     -- Committee Size - No Encrypted Votes
 0                      -- Committee Threshold - No Encrypted Votes
 );


-- Free large binary file contents
\unset block0contents

--sql
-- Challenges for Fund 3
INSERT INTO objective
(
    id,
    event,
    category,
    title,
    description,
    rewards_currency,
    rewards_total,
    rewards_total_lovelace,
    proposers_rewards,
    vote_options,
    extra)
VALUES

(
    1, -- Objective ID
    4, -- event id
    'catalyst-community-choice', -- category
    'Fund6 Challenge Setting', -- title
    'What Challenges should the community prioritize to address in Fund6?', -- description
    'USD_ADA', -- Currency
    400000, -- rewards total
    NULL, -- rewards_total_lovelace
    0, -- proposers rewards
    1, -- vote_options
    '{"url": {"objective": "https://cardano.ideascale.com/a/campaign-home/25874"}}' -- extra objective data
)
,

(
    2, -- Objective ID
    4, -- event id
    'catalyst-simple', -- category
    'Dapps & Integrations', -- title
    'How can application developers drive adoption of Cardano in 2021?', -- description
    'USD_ADA', -- Currency
    200000, -- rewards total
    NULL, -- rewards_total_lovelace
    200000, -- proposers rewards
    1, -- vote_options
    '{"url": {"objective": "https://cardano.ideascale.com/a/campaign-home/25869"}}' -- extra objective data
)
,

(
    3, -- Objective ID
    4, -- event id
    'catalyst-simple', -- category
    'Developer Ecosystem', -- title
    'How can we create a positive developer experience that helps the developer focus on building successful apps?', -- description
    'USD_ADA', -- Currency
    400000, -- rewards total
    NULL, -- rewards_total_lovelace
    400000, -- proposers rewards
    1, -- vote_options
    '{"url": {"objective": "https://cardano.ideascale.com/a/campaign-home/25868"}}' -- extra objective data
)
,

(
    4, -- Objective ID
    4, -- event id
    'catalyst-simple', -- category
    'Distributed Decision Making', -- title
    'How can we help the Catalyst community to get better at distributed decision making within the next two Catalyst rounds?', -- description
    'USD_ADA', -- Currency
    50000, -- rewards total
    NULL, -- rewards_total_lovelace
    50000, -- proposers rewards
    1, -- vote_options
    '{"url": {"objective": "https://cardano.ideascale.com/a/campaign-home/25870"}}' -- extra objective data
)
,

(
    5, -- Objective ID
    4, -- event id
    'catalyst-simple', -- category
    'Proposer Outreach', -- title
    'How can we encourage entrepreneurs from outside the Cardano ecosystem to submit proposals to Catalyst in the next two funds?', -- description
    'USD_ADA', -- Currency
    50000, -- rewards total
    NULL, -- rewards_total_lovelace
    50000, -- proposers rewards
    1, -- vote_options
    '{"url": {"objective": "https://cardano.ideascale.com/a/campaign-home/25871"}}' -- extra objective data
)
,

(
    6, -- Objective ID
    4, -- event id
    'catalyst-simple', -- category
    'Catalyst Value Onboarding', -- title
    'How can we encourage more meaningful participation in Project Catalyst from community members in the next two funds?', -- description
    'USD_ADA', -- Currency
    50000, -- rewards total
    NULL, -- rewards_total_lovelace
    50000, -- proposers rewards
    1, -- vote_options
    '{"url": {"objective": "https://cardano.ideascale.com/a/campaign-home/25872"}}' -- extra objective data
)
,

(
    7, -- Objective ID
    4, -- event id
    'catalyst-simple', -- category
    'Local Community Centers', -- title
    'How can we mobilize the community to solve local problems using Cardano, through a Local Community Center model supported by the CF?', -- description
    'USD_ADA', -- Currency
    50000, -- rewards total
    NULL, -- rewards_total_lovelace
    50000, -- proposers rewards
    1, -- vote_options
    '{"url": {"objective": "https://cardano.ideascale.com/a/campaign-home/25873"}}' -- extra objective data
)

;

--sql
-- All Proposals for  FUND 2
INSERT INTO proposal
(
    id,
    objective,
    title,
    summary,
    category,
    public_key,
    funds,
    url,
    files_url,
    impact_score,
    extra,
    proposer_name,
    proposer_contact,
    proposer_url,
    proposer_relevant_experience,
    bb_proposal_id,
    bb_vote_options
)
VALUES

(
    0,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Decentralized Football Betting Pool',  -- title
    'Football betting pools face limitations due to lack of transparency, inconvenient buy-ins/payouts, and require trust in a centralized party.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'yt21KAeffeVCftaX4crEIkOWLxWrJczmsBkOWRztLiE=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBfWb', -- url
    '', -- files_url
    276, -- impact_score
    '{"solution": "I will create a dapp that will allow casual fans to easily create and participate in transparent football betting pools of various prices."}', -- extra
    'jcraney', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I am a professional TypeScript developer working in the banking industry.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    1,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=4), -- objective
    'Migrate Ethereum',  -- title
    'How can we get more Ethereum projects to start moving to Cardano in the next 3 months?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=4), -- category - VITSS Compat ONLY
    'uL9PFhb9XpS21GSbCV6B+EQT+kN3I0PaJcMIy8ATX98=', -- Public Payment Key
    '400000', -- funds
    'http://ideascale.com/t/UM5UZBfWi', -- url
    '', -- files_url
    319, -- impact_score
    '{"brief": "Creating an environment where Ethereum projects intend, can and do migrate to Cardano will have a snowball effect that drives other Ethereum projects to migrate to Cardano.  \r\nBudget  \r\nUS$400,000 in ada  \r\n(Personally, I think the max should be $500,000 as it is a number that sparks more interest)  \r\nWhat does success look like  \r\nThe Ethereum community and crypto media are talking regularly about projects that are moving or have moved successfully to Cardano.  \r\nEthereum projects that are considering moving feel fully informed about the benefits, challenges, risks and drawbacks of migrating.  \r\nSignificant Ethereum projects are regularly announcing and/or commencing their migration to Cardano.  \r\nMigrating Ethereum projects to Cardano is considered easy, effective and well supported  \r\nGuiding questions\r\n\r\n*   What documentation and tools do Ethereum projects need to migrate\r\n*   How do we attract Ethereum projects to migrate to Cardano and/or participate in Catalyst?\r\n*   What do Ethereum Projects want, and how do they get it from the Cardano dev ecosystem?  \r\n    \r\n\r\nPotential directions  \r\n\r\n*   Start a migration from Ethereum to Cardano\r\n*   Developer productivity: IDE''s, scripts to automate stuff.\r\n*   Knowledge base & Documentation\r\n*   Deployment, testing, and monitoring frameworks\r\n*   Samples, recipes and templates\r\n*   Hackathons\r\n*   API''s, and oracles.", "importance": "Ethereum has major projects and problems. Migration to Cardano would be good for the projects and Cardano. Ethereum 2.0 has a long way to go", "goal": "The Ethereum community and crypto media are talking regularly about projects that are moving or have moved successfully to Cardano.", "metrics": "*   Number of announced migrations\r\n*   Number of commenced migrations\r\n*   Number of completed migrations\r\n*   Number of abandoned or failed migrations"}', -- extra
    'Greg Bell', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    2,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Layer 2 Advanced Architecture ',  -- title
    'Cardano NEEDS a cutting edge layer 2 data solution that will handle large volumes of, and high rates of, irregular and complex data.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    '5Q6FeVKYnT9DGMfHOBvKFBbwd1nt2Y8DE7uD30cCIio=', -- Public Payment Key
    '200000', -- funds
    'http://ideascale.com/t/UM5UZBfWm', -- url
    '', -- files_url
    240, -- impact_score
    '{"solution": "A Decentralized AI System as a not-for-profit that will serve the community and charity."}', -- extra
    'drakemonroe', -- proposer name
    '', -- proposer contact
    'https://discord.gg/w9XV8WMbGq', -- proposer URL
    'Cardano Wise''rd. I''ve asked Ben Goertzel to help me train the system and IOHK to make the software mods.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    3,  -- id
    (SELECT row_id FROM objective WHERE id=4 AND event=4), -- objective
    'Idea: decentralized hierarchies',  -- title
    'It''s hard to determine the best way to distribute power, hierarchies are necessary but it''s almost impossible to make them right.',  -- summary
    (SELECT category FROM objective WHERE id=4 AND event=4), -- category - VITSS Compat ONLY
    'iISngZhavv46wouMA/1TEIWXz4cqoMxoW4g+kBvoixU=', -- Public Payment Key
    '800', -- funds
    'http://ideascale.com/t/UM5UZBfWr', -- url
    '', -- files_url
    236, -- impact_score
    '{"solution": "A design for the development of a system that can create many different types of power structures and let people try them to see what works."}', -- extra
    'F', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I''m just a self taught programmer, I''ve been working as a freelancer for 3 years now and I believe I had a very good career so far.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    4,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Documentation Alignment',  -- title
    'There isn''t a clear strategy for getting up to speed as a developer new to Cardano',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'rDFtjn4nMgvbknUayB9PqV3aWbcMT2WTuH07sC5RMOY=', -- Public Payment Key
    '5500', -- funds
    'http://ideascale.com/t/UM5UZBfW0', -- url
    '', -- files_url
    290, -- impact_score
    '{"solution": "Group development resources by their audience under single source  \r\nProvide consistent information across documentation"}', -- extra
    'greerben0', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Enterprise software engineer, software engineering team lead, and consulting team lead.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    5,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Cardano-based Marketplace',  -- title
    'A user can easily create a token but does not have any way to sell their token besides peer-to-peer trading.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'aP4/tZhmGUapKtDGdGS9NyasUZN4PbS5KPrTLjFK15s=', -- Public Payment Key
    '5000', -- funds
    'http://ideascale.com/t/UM5UZBfW3', -- url
    '', -- files_url
    340, -- impact_score
    '{"solution": "The marketplace will allow any user to sell any verifiable token on the Cardano blockchain using smart contracts written in Plutus."}', -- extra
    'quinn', -- proposer name
    '', -- proposer contact
    'https://github.com/logicalmechanism', -- proposer URL
    'I am a theoretical astrophysicist who transitioned careers to become a dApp developer on the Cardano blockchain. www.logicalmechanism.io', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    6,  -- id
    (SELECT row_id FROM objective WHERE id=5 AND event=4), -- objective
    'West Africa Proposer Outreach',  -- title
    'West African entrepreneurs are unaware of the Project Catalyst''s funding campaigns and the community''s encouragement of African proposals',  -- summary
    (SELECT category FROM objective WHERE id=5 AND event=4), -- category - VITSS Compat ONLY
    'dduuT+pOkw+H0pN0HFqBG6HkW3IhnztMyr7kaKV4BxY=', -- Public Payment Key
    '6000', -- funds
    'http://ideascale.com/t/UM5UZBfXD', -- url
    '', -- files_url
    463, -- impact_score
    '{"solution": "A 2-Day Virtual Event with guest speakers, project proposals, interviews, live Q&A and 2x1-hour webinar workshops (French & English)"}', -- extra
    'WADA(West Africa Decentralized Alliance)', -- proposer name
    '', -- proposer contact
    'http://bit.ly/3ciLnmz', -- proposer URL
    'Team: Training, Consultancy, Project Management, Community Engagement & Outreach', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    7,  -- id
    (SELECT row_id FROM objective WHERE id=7 AND event=4), -- objective
    'Local Centers in West Africa',  -- title
    'WADA''s *Blockchain* *Resource Hubs for Solution Design* have no formal relationship with the Cardano Foundation (CF).',  -- summary
    (SELECT category FROM objective WHERE id=7 AND event=4), -- category - VITSS Compat ONLY
    'gOLM7LmhTkjjSNQGr9euLNBf6c2xacPjksqySkRKJNQ=', -- Public Payment Key
    '4000', -- funds
    'http://ideascale.com/t/UM5UZBfXP', -- url
    '', -- files_url
    476, -- impact_score
    '{"solution": "Obtain Cardano Foundation funding to register Hubs, and to enable access to the Foundation''s oversight and resources."}', -- extra
    'WADA(West Africa Decentralized Alliance)', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Team: Marketing, Project & Nonprofit Work Experience in W/Africa, Software Developers, Project Management, Analytics, Community Engagement', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    8,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    '+ Dev incentives in Cameroon',  -- title
    'Need welcoming space for developers to come together and brainstorm solutions (on a community level) built on Distributed Ledger Technology.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'ly9xZ39xFGhCG0yV2EATj2QqptGTgPR52VNPSdwwLAE=', -- Public Payment Key
    '9000', -- funds
    'http://ideascale.com/t/UM5UZBfXU', -- url
    '', -- files_url
    356, -- impact_score
    '{"solution": "Offer unique gateway into the world of DLT, incentivizing Hub attendance and participation by providing above and beyond community support."}', -- extra
    'Megan Hess', -- proposer name
    '', -- proposer contact
    'http://www.deweycameroon.net/', -- proposer URL
    'Local Team: Skilled bilingual (English/French) software developers and educators (STEM), hospitality, outreach, and marketing specialists', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    9,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Developer Library',  -- title
    'Developers have limited time to learn. Why not create a library of learning resources where each developer has a learning plan and a coach.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    '0tPe7BUYZ7NnryG2dcee1JrYlE3iu5fsyW35oFIdDmY=', -- Public Payment Key
    '15000', -- funds
    'http://ideascale.com/t/UM5UZBfXu', -- url
    '', -- files_url
    200, -- impact_score
    '{"solution": "Create a one stop shopping solution. A library where each developer can be coached on a learning plan by a qualified instructor. Teamwork!"}', -- extra
    'jeffg', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Managing for federal public service managing teams of diverse work groups, learning plans and coaches are effective 4 achieving goals.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    10,  -- id
    (SELECT row_id FROM objective WHERE id=4 AND event=4), -- objective
    'Working Groups',  -- title
    'We need to assemble talent in groups to then assign workflow and create synergies and efficiencies.',  -- summary
    (SELECT category FROM objective WHERE id=4 AND event=4), -- category - VITSS Compat ONLY
    'nYTPG1HpCDct7fG81gzuLIHjGewcMjp6FcUTGAfVDSA=', -- Public Payment Key
    '10000', -- funds
    'http://ideascale.com/t/UM5UZBfXv', -- url
    '', -- files_url
    120, -- impact_score
    '{"solution": "Create professional talent pools that can assist projects with required tasks and the pool can share in the rewards."}', -- extra
    'jeffg', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Grouping talent together creates synergy and tasking/monitoring by the group and collaboration create efficiencies.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    11,  -- id
    (SELECT row_id FROM objective WHERE id=5 AND event=4), -- objective
    'University/College Outreach',  -- title
    'Solicit the youngest and brightest from around the globe to contribute to the network. Raising awareness with student organizations.',  -- summary
    (SELECT category FROM objective WHERE id=5 AND event=4), -- category - VITSS Compat ONLY
    'pgsK+S4YlqnB1LMqFlZqdDDIDfAEsyJQ4DKyKDaKW88=', -- Public Payment Key
    '10000', -- funds
    'http://ideascale.com/t/UM5UZBfXz', -- url
    '', -- files_url
    267, -- impact_score
    '{"solution": "Reach student associations and faculties to raise awareness. Social media blitz combined with newsletters and outreach activities to engage."}', -- extra
    'jeffg', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Brought regional and national offices together via working groups and through analytics to create results while having accountability.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    12,  -- id
    (SELECT row_id FROM objective WHERE id=6 AND event=4), -- objective
    'Create Social Network in Catalyst',  -- title
    'Create a social network so participants can like and work with each other and shares ideas and collaborate.',  -- summary
    (SELECT category FROM objective WHERE id=6 AND event=4), -- category - VITSS Compat ONLY
    '+6Wh5TmSaDsKxVz+76Ec2Gh3JP2z0d0LrjeuDEoGwcI=', -- Public Payment Key
    '5000', -- funds
    'http://ideascale.com/t/UM5UZBfX1', -- url
    '', -- files_url
    119, -- impact_score
    '{"solution": "Allowing people to share and work on ideas within catalyst and allow for the development of the social network."}', -- extra
    'jeffg', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'As a partner in a dental whitening business we collected data from clients and encourage referrals and repeat visits.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    13,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=4), -- objective
    'Clean Water',  -- title
    'How can we encourage ADA owners to donate to clean water projects around the world.',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=4), -- category - VITSS Compat ONLY
    'LjEpWzxyT2TzbLCls0Lx2GpXkSc7dGOi2xL8TetKaew=', -- Public Payment Key
    '5000', -- funds
    'http://ideascale.com/t/UM5UZBfX4', -- url
    '', -- files_url
    229, -- impact_score
    '{"brief": "A pledge campaign seeking 1% contribution from ADA rewards to the clean water project.", "importance": "We need to create social responsibility within the network to help those that need help for basic resources.", "goal": "Having ADA holders donate a portion of their rewards to clean water projects. Raising the awareness of regional disparities and global issue", "metrics": "Number of projects funded, completed and being maintained."}', -- extra
    'jeffg', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    14,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Nada: Nodes-as-a-Service',  -- title
    'Zero config, instant Cardano node access for free.  
In a similar spirit to Infura.io, access to nodes should be free & easy.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'qqZ8S9dj9eaLyc5Uj3cx0kYhA91Fxzy3igIU4ZBrAGQ=', -- Public Payment Key
    '40000', -- funds
    'http://ideascale.com/t/UM5UZBfYX', -- url
    '', -- files_url
    180, -- impact_score
    '{"solution": "A backend platform would maintain many horizontally-scalable pools of nodes. Access to these would be controlled via API keys & accounts."}', -- extra
    'Thomas Ruble', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I am a cloud DevOps engineer at Google, with a specialization in automation & enterprise scaling.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    15,  -- id
    (SELECT row_id FROM objective WHERE id=6 AND event=4), -- objective
    'Encourage proposals',  -- title
    'Proposers play an all or nothing role and they can''t vote.',  -- summary
    (SELECT category FROM objective WHERE id=6 AND event=4), -- category - VITSS Compat ONLY
    'lGQ+1vazjwT2yKkvJEMSzuRoOOxi5DNb2Kl6lxvWpr4=', -- Public Payment Key
    '1', -- funds
    'http://ideascale.com/t/UM5UZBfYl', -- url
    '', -- files_url
    189, -- impact_score
    '{"solution": "Make a rewards pool for proposers that participate until the end of the process to encourage them to participate"}', -- extra
    'Jean', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I participate to fund3', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    16,  -- id
    (SELECT row_id FROM objective WHERE id=4 AND event=4), -- objective
    'Guides to project Catalyst success',  -- title
    'Catalyst is a confusing place when you are new to it. New users don''t have a really concise guide to doing well on Project Catalyst',  -- summary
    (SELECT category FROM objective WHERE id=4 AND event=4), -- category - VITSS Compat ONLY
    'SxA7BBKgJATJW0nArOvKKtAIAAKUWMK93xB9hEztvNA=', -- Public Payment Key
    '100', -- funds
    'http://ideascale.com/t/UM5UZBfYw', -- url
    '', -- files_url
    280, -- impact_score
    '{"solution": "Create concise guides to being successful on project Catalyst for all (enthusiasts, developers, entrepreneurs, community advisors, etc)"}', -- extra
    'Greg Bell', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I spend too much time lurking on project Catalyst and when I acted, I made a lot of mistakes to learn from.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    17,  -- id
    (SELECT row_id FROM objective WHERE id=6 AND event=4), -- objective
    'West Africa Catalyst Onboarding',  -- title
    'No streamlined framework to onboard talented West Africans making contact through Cardano websites/social media/groups into Project Catalyst',  -- summary
    (SELECT category FROM objective WHERE id=6 AND event=4), -- category - VITSS Compat ONLY
    'lRBxV2/sExdhsxGFyC58SzBfQLv1+DUTVkEFp0vVkrM=', -- Public Payment Key
    '6000', -- funds
    'http://ideascale.com/t/UM5UZBfY3', -- url
    '', -- files_url
    407, -- impact_score
    '{"solution": "Implement a diversified membership scheme to attract and engage W/Africans to contribute to Project Catalyst goals"}', -- extra
    'WADA(West Africa Decentralized Alliance)', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Team: Marketing, Educator, Community Outreach, Project Management, Software Developers. Research & Development Consultancy', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    18,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=4), -- objective
    'DeFi and Microlending for Africa',  -- title
    'How can we enable the creation of micro-lending and Defi dApp solutions that fits the African setting?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=4), -- category - VITSS Compat ONLY
    '5o1rc/Z+swlHCwA1j3yAmki8OD31GRWydQCABzJPlPY=', -- Public Payment Key
    '90000', -- funds
    'http://ideascale.com/t/UM5UZBfaC', -- url
    '', -- files_url
    377, -- impact_score
    '{"brief": "++Sources:++  \r\n1\\. Why conventional banking is not working for sub-Saharan Africa: https://www.imf.org/external/pubs/ft/wp/2004/wp0455.pdf  \r\n2\\. How mobile money and other Startups are solving the African banking crises although financial inclusion remains low: https://www.researchgate.net/publication/316847980_The_Rise_of_Financial_Services_in_Africa_An_Historical_Perspective_The_Importance_of_High-Impact_Entrepreneurship  \r\n3\\. How mobile phones uniquely evolved to become the banking solution in East Africa: https://youtu.be/1O83CnrKfkk  \r\n4\\. Financial inclusion in Africa: https://www.afdb.org/fileadmin/uploads/afdb/Documents/Project-and-Operations/Financial_Inclusion_in_Africa.pdf  \r\n5\\. How Africa''s ancient practices are being digitized for the future: https://qz.com/africa/1392527/africas-ancient-practices-are-being-digitized-for-the-future", "importance": "Microlending and DeFi dApps targeted specifically to the African context & based on historical cultural norms will accelerate user adoption", "goal": "Businesses are confident to submit well-researched proposals that address the African micro-lending and decentralized finance context", "metrics": "*   Submitted proposals well researched, viable and addresses traditional African financial cultural norms and practices\r\n*   3 or more successfully funded micro-lending and Defi dApp launched within the next 3 funding rounds\r\n*   Minimum 50% user adoption in funded proposal''s target market within 2 years"}', -- extra
    'WADA(West Africa Decentralized Alliance)', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    19,  -- id
    (SELECT row_id FROM objective WHERE id=7 AND event=4), -- objective
    'ADA in CIS exchangers/online stores',  -- title
    'Every user that spends $1000 on cryptocurrency purchases lost about $15-35 as a stock exchange commission https://imgur.com/a/Lk7oHga',  -- summary
    (SELECT category FROM objective WHERE id=7 AND event=4), -- category - VITSS Compat ONLY
    'y5mS1oRx2hXdvjrl3T36/aGzKdpvsMf0DaU3Z/HArAc=', -- Public Payment Key
    '5000', -- funds
    'http://ideascale.com/t/UM5UZBfaD', -- url
    '', -- files_url
    275, -- impact_score
    '{"solution": "We need to convince each exchanger and online store to use ADA on their site. It''ll allow to SAVE $15-30 for every $1000 spent."}', -- extra
    'Oleksii', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I had experience working in a call center of the bank (consultations, sales, and advertising of new products), so I can handle this task.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    20,  -- id
    (SELECT row_id FROM objective WHERE id=7 AND event=4), -- objective
    'Local Venture Activation Centres',  -- title
    'Good people do good but most don''t get support, connections and mentorship vital to starting and sustainably scaling their aspirations.',  -- summary
    (SELECT category FROM objective WHERE id=7 AND event=4), -- category - VITSS Compat ONLY
    '1J3CxYqPaI67fQARBSxme+Ivah+7naAyKBPLf84gEnY=', -- Public Payment Key
    '30000', -- funds
    'http://ideascale.com/t/UM5UZBfaH', -- url
    '', -- files_url
    411, -- impact_score
    '{"solution": "An activation programme focused on delivering \"watch-try-teach\" community-led innovation in UN SDGs and multicapital accounting."}', -- extra
    'jo allum', -- proposer name
    '', -- proposer contact
    'https://www.venturecentre.nz/', -- proposer URL
    'Built a community-led activation programme over seven years realised in a network of thirty centres. Involved with Reporting 3.0 r3-0.org.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    21,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Demos DAO',  -- title
    'Current political parties and politicians are having troubles with serving their voters and keeping their integrity.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'XNRLkI2wwcxSi5JxX6n3rQ3jGvWeclwSd2V8rmp1L7A=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBfaP', -- url
    '', -- files_url
    288, -- impact_score
    '{"solution": "DAO decision making political system based on Cardano to provide transparency, security and anonymity of voting designed to run on elections"}', -- extra
    'F', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Team of an experienced product owner, CEO of an IT company which has a developed dashboard, 5 developers for integration, volounteers', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    22,  -- id
    (SELECT row_id FROM objective WHERE id=7 AND event=4), -- objective
    'Cardano Center in Europe',  -- title
    'There is no place in Europe where developers, entrepreneurs and other community members can meet to learn and work on Cardano projects.',  -- summary
    (SELECT category FROM objective WHERE id=7 AND event=4), -- category - VITSS Compat ONLY
    'jqKQGk1c355f9PaQEkDGnMz1oYcYR0+9NQx/y+nJdyE=', -- Public Payment Key
    '31800', -- funds
    'http://ideascale.com/t/UM5UZBfaW', -- url
    '', -- files_url
    157, -- impact_score
    '{"solution": "We want to develop a Cardano Center with coworking space and activities (meetups, demos, brainstorms, workshops and training) in Amsterdam."}', -- extra
    'Ryan Morrison', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '13 years experience in marketing for tech companies

Experience in blockchain, education and community building (host Cardano Podcast)', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    23,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Auto-Paying Royalty System',  -- title
    'Creators of creative content are not properly paid royalties: there are no means of verifiably tracking royalty payouts.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'bDmUrYrvWHa4iyzwWfPpj0Hcu4DXacamE4WMnHNWxAg=', -- Public Payment Key
    '75000', -- funds
    'http://ideascale.com/t/UM5UZBfah', -- url
    '', -- files_url
    229, -- impact_score
    '{"solution": "A media blockchain that is pay per play, and pays royalties direct to the authors/owners of the content, or as appropriated by media type."}', -- extra
    'gabetcras', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '15+ years in the audio industry, struggling to monetize assets.

Educated in Music Business and Audio Engineering

2+ years on ETH contracts', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    24,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'The Cardano Global Treasure Hunt',  -- title
    'More people need to discover Cardano. Create a desire for them to engage in a fun way by organising a global event with dapps to support it.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    '2TIrQKj7qY8QTKxzB51yhOJ3ksCNsu56u+Ta/vLlvIA=', -- Public Payment Key
    '30680', -- funds
    'http://ideascale.com/t/UM5UZBfam', -- url
    '', -- files_url
    367, -- impact_score
    '{"solution": "Attract new users to the Cardano ecosystem by harnessing the ingenuity of its global community to organise a unique, fun & rewarding event."}', -- extra
    'newmindflow', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'NewMindflow - award-winning film directors, game designers, visual artists & art app devs. Entrepreneurs, founders of New Mindflow Studio.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    25,  -- id
    (SELECT row_id FROM objective WHERE id=7 AND event=4), -- objective
    'Cardano spanish training programs',  -- title
    'There are no educational resources for spanish speakers who want to develop and grow their Cardano local community.',  -- summary
    (SELECT category FROM objective WHERE id=7 AND event=4), -- category - VITSS Compat ONLY
    'eacfI+/6QHfcdLBg6yQUvsWae65rVCspfyPf2yQngGs=', -- Public Payment Key
    '16900', -- funds
    'http://ideascale.com/t/UM5UZBfat', -- url
    '', -- files_url
    413, -- impact_score
    '{"solution": "Create an introductory Cardano spanish training program to understand its basics, and also a website to incentivize community engagement"}', -- extra
    'Tomas Garro', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Team: Community management, entrepreneurs, website developers.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    26,  -- id
    (SELECT row_id FROM objective WHERE id=7 AND event=4), -- objective
    'Catalyst Legal Fund',  -- title
    'If we are doing our job, we''ll be pushing new boundaries and proposers need a legal point of contact.',  -- summary
    (SELECT category FROM objective WHERE id=7 AND event=4), -- category - VITSS Compat ONLY
    '3bGui1KwrMpOGo1k+TDn/ANSzhRgearXOHPni1xWGqA=', -- Public Payment Key
    '15000', -- funds
    'http://ideascale.com/t/UM5UZBfa6', -- url
    '', -- files_url
    125, -- impact_score
    '{"solution": "The CF takes the proposal amount and sets up legal fund for proposers and cohorts."}', -- extra
    'drakemonroe', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'On CF to establish fund.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    27,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Xceed Decentralized eLearning',  -- title
    'Current e-Learning Platforms are Centralized, Prescriptive and not tailored to the African Situation',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'xX/qsVSwYy0YsZeChNRZK9JzrjxIsvJDlo5r94EwA2k=', -- Public Payment Key
    '10000', -- funds
    'http://ideascale.com/t/UM5UZBfbC', -- url
    '', -- files_url
    333, -- impact_score
    '{"solution": "Bring Governance, Decentralization and Incentives to e-Learning (A decentralized Udemy)"}', -- extra
    'Chuma Chukwujama', -- proposer name
    '', -- proposer contact
    'https://xceed365.com', -- proposer URL
    'XceedNetwork has pioneered Human Capital management technology and e-learning platforms in Africa for over 15 years', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    28,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Peer-to-peer Cryptocurrency Market',  -- title
    'Purchasing cryptocurrencies can be a complicated process, especially for people that do not live in first world countries.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'rCRSF60Iuz4LXBycHa+NBpsQ96qCgW80iVdTB2OFo5o=', -- Public Payment Key
    '6000', -- funds
    'http://ideascale.com/t/UM5UZBfbK', -- url
    '', -- files_url
    320, -- impact_score
    '{"solution": "A peer-to-peer non-custodial decentralized exchange, allowing for fiat-to-crypto and crypto-to-crypto trading, built on the right platform."}', -- extra
    'Rene Vergara', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'IT consulting, IT Product management in Fortune 50 company, Java programming.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    29,  -- id
    (SELECT row_id FROM objective WHERE id=5 AND event=4), -- objective
    'Sustainable ADA',  -- title
    'Sustainable ADA will expand outreach and education for explaining use cases/examples of how Cardano can help create a sustainable world.',  -- summary
    (SELECT category FROM objective WHERE id=5 AND event=4), -- category - VITSS Compat ONLY
    '+yoddOzUJis4nZxPgAZrienAI3NW70xgu+n01ki9YQ8=', -- Public Payment Key
    '44000', -- funds
    'http://ideascale.com/t/UM5UZBfby', -- url
    '', -- files_url
    213, -- impact_score
    '{"solution": "The goal is to connect aspects of Cardano to our current SDGs creating an equal and sustainable world with this blockchain technology."}', -- extra
    'cole.vt', -- proposer name
    '', -- proposer contact
    'http://sustainableada.com', -- proposer URL
    'I am a senior at the University of New Hampshire an Economics Major, and a dual major in Sustainability. Experienced Sustainability research', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    30,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=4), -- objective
    'Cardano Web Hub',  -- title
    'How can Cardano establish a more meaningful and impactful presence on the internet to reach out general public and community members ?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=4), -- category - VITSS Compat ONLY
    'N13lZ6yMLso68jlnAQS24J8VeXzB5ZLlSuiyyXSqemI=', -- Public Payment Key
    '25000', -- funds
    'http://ideascale.com/t/UM5UZBfcA', -- url
    '', -- files_url
    296, -- impact_score
    '{"brief": "To create a landing page and gateway to the Cardano World. A place where anyone can easily be introduced to the blockchain technology in general, its roots & history, where we stand and what''s into the future, putting Cardano into context.\r\n\r\nWe need a true and elegant, beautifully designed multilingual gateway to the Ecosystem, enhancing Cardano\u00b4s branding, highlighting its mission and vision, underlining the unique development concepts, aggregating Cardano'' s immense online resources , channels and partnerships.", "importance": "A global multilingual Cardano Web Hub is critical for an easy introduction and access to the ecosystem\u00b4s online resources.", "goal": "Web designers submit proposals for Web Hub in visual /interactive mode, addressing challenge brief.", "metrics": "Minimum 10 proposals.\r\n\r\nWinner ability to implement the proposal with in 30 days of award."}', -- extra
    'Ronin', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    31,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=4), -- objective
    'Success Tracking',  -- title
    'How can we build and maintain a better success tracking experience for funded proposals?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=4), -- category - VITSS Compat ONLY
    '+dTm4VvXFXPEp4MJHGtfGhTJXUvvyl0RuNFvh3CXzN0=', -- Public Payment Key
    '100000', -- funds
    'http://ideascale.com/t/UM5UZBfcP', -- url
    '', -- files_url
    396, -- impact_score
    '{"brief": "Informative! A thriving ecosystem of well-informed community members equipped with tools that enable access to updates on Programs/Dapps being funded through Project Catalyst. Accomplished! Seeing the data on projects flourishing and knowing we were part of that solution brings everything into reality and helps unify our efforts. Utilized!  \r\nWell after voting, many community members may want to stay up to date with the progress of funded proposals. It may be important that we have well laid out information as to how these projects are doing and ways we can learn from their success (or lack of).  \r\nGuiding questions  \r\n\u2022 What are some metrics the community might find useful about funded proposals?  \r\n\u2022 How will people access this information?  \r\n\u2022 How can this information be displayed?  \r\nPotential directions  \r\n\u2022 Software that displays updates on funded proposals  \r\n\u2022 Web based environment", "importance": "Knowledge of the progress on funded proposals can serve useful as a community reference, help determine ROI, and help with adoption.", "goal": "A thriving ecosystem of well-informed community members ready to answer any questions they may receive about a catalyst funded project.", "metrics": "At the end of this challenge, we will be asking ourselves: Did we manage to create a better experience for community members to track the success of catalyst funded projects?\r\n\r\n\u2022 Quality and efficacy of tool(s) being utilized for information about projects funded.\r\n\r\n\u2022 Tool(s) engagement with the community at large. (views, clicks, shares, etc\u2026)"}', -- extra
    'Carlos Hernandez', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    32,  -- id
    (SELECT row_id FROM objective WHERE id=4 AND event=4), -- objective
    'Meta-Memory (Fmrly Fed. Debate)',  -- title
    'Debates and discussion are marred by talking past one another and pushing misinformation or logical fallacies, often obscured or hidden.',  -- summary
    (SELECT category FROM objective WHERE id=4 AND event=4), -- category - VITSS Compat ONLY
    '1LRdyzxcBvDX5XWjaFL6RgbWdQS06+hgh2FyvCO52v4=', -- Public Payment Key
    '5000', -- funds
    'http://ideascale.com/t/UM5UZBfcb', -- url
    '', -- files_url
    414, -- impact_score
    '{"solution": "Break ideas/statements into digestible parts. Debate each sub-point, voting, logic analysis, idea linking, commenting, and meta-analysis."}', -- extra
    'Callie Bolton', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Attorney 10 years, skilled in the art of persuasion and logical analysis.  
Project manager, telecom, 5 years+  
Rudimentary coding background', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    33,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Littercoin - Mass Adoption',  -- title
    'Plastic pollution is a huge global problem but data is lacking. Littercoin can incentivise data collection while increasing Cardano adoption',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'D1LDkkvbtxxvb2q24oWyreq9ELcV/BZidrph8skY5CU=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBfcs', -- url
    '', -- files_url
    444, -- impact_score
    '{"solution": "OpenLitterMap.com rewards users with Littercoin by applying proof of work principles to citizen science for the first time."}', -- extra
    'Sean Lynch', -- proposer name
    '', -- proposer contact
    'https://github.com/openlittermap', -- proposer URL
    'Started researching litter mapping in 2008. Worked as a divemaster in the tropics. Did x2 MSc on the methodology then taught myself how2code', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    34,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Peer-to-peer Science ',  -- title
    'Universities are oversized, inefficient middlemen. The direct transfer of funds from funders to scientists can be enabled by smart contracts',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'JXY53ZykaQB9laAGtrf1usQAVPqR5hPxbI6qKho9+eE=', -- Public Payment Key
    '100000', -- funds
    'http://ideascale.com/t/UM5UZBfcv', -- url
    '', -- files_url
    233, -- impact_score
    '{"solution": "A trust-less, smart contract system which allows funders to entrust their funds directly with scientists without University middlemen."}', -- extra
    'Ali Ghareeb', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I am a published scientist with experience in raising funds for research and a programmer.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    35,  -- id
    (SELECT row_id FROM objective WHERE id=7 AND event=4), -- objective
    'Charity Casino',  -- title
    'People want to make change but don''t have the skills, finances or time to do it. How can we help hubs flourish?',  -- summary
    (SELECT category FROM objective WHERE id=7 AND event=4), -- category - VITSS Compat ONLY
    'VsKk/j4HV7MFLEAr32yW3T+v+SeGp6EcodZhhZF2MX4=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBfc7', -- url
    '', -- files_url
    145, -- impact_score
    '{"solution": "Every 5 days a winning donator will be picked.  \r\nWinner receives:  \r\n1) 1% of epochs staking rewards  \r\n2) 4% rewards to hub/charity of his choice"}', -- extra
    'brian.bxter', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'My partner has a degree in computer science and multi years experience in java. I have experience managing a business.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    36,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Influencer Marketing Smart Contract',  -- title
    'Influencer marketing is a rapidly growing industry with thousands of financial transactions being executed manually every day.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    't2REnoEZZH8UqtW/zmvUsGB5C2tz4+k5EH3FJZOlQlU=', -- Public Payment Key
    '10000', -- funds
    'http://ideascale.com/t/UM5UZBfdK', -- url
    '', -- files_url
    180, -- impact_score
    '{"solution": "We are creating a platform where companies and influencers can easily create smart contracts on the Cardano Blockchain."}', -- extra
    'jschreiner22', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'We have three programmers. Two have front-end experience, two have back-end experience, and one has limited experience with Ethereum dapps.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    37,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Sports Industry Smart Contracts',  -- title
    'Front offices of sports franchises are inefficient. They take weeks/months to create/negotiate contracts and forget player payments.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'edb6ZpxsHIOiNBeM82t+wat3cGzVTSRD0drn4WoH+bk=', -- Public Payment Key
    '10000', -- funds
    'http://ideascale.com/t/UM5UZBfdM', -- url
    '', -- files_url
    183, -- impact_score
    '{"solution": "We want to integrate Cardano smart contracts into the sports industry and carry out payments automatically with trusted data."}', -- extra
    'Logan Panchot', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I am a professional soccer player that knows this is a problem and am part of a team of 4 developers with the needed skillsets for this idea', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    38,  -- id
    (SELECT row_id FROM objective WHERE id=5 AND event=4), -- objective
    'Proposer to Producer Demo',  -- title
    'The step-by-step process of proposing a project, getting it funded and applying it in the real world is not clear and accessible.',  -- summary
    (SELECT category FROM objective WHERE id=5 AND event=4), -- category - VITSS Compat ONLY
    'KJjKot/xa5bbrvIsFMeuYqQX//ca4KMye4acliOKRJw=', -- Public Payment Key
    '30000', -- funds
    'http://ideascale.com/t/UM5UZBfdV', -- url
    '', -- files_url
    267, -- impact_score
    '{"solution": "Provide a clear and accessible multimedia demonstration of a mini project from proposal to real word application."}', -- extra
    'liamcardano', -- proposer name
    '', -- proposer contact
    'http://vidintro.com', -- proposer URL
    'I am a videographer in the early stages of starting my own video production company which aims to connect small business with videographers.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    39,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Global Restaurant Dapp - BIBOP',  -- title
    'Covid-19 changed the way we dine in at restaurants worldwide.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    '2QlkL+8EiBGAWHZB8axI1FyMrk5+Mc3O1NLCcp8en20=', -- Public Payment Key
    '10000', -- funds
    'http://ideascale.com/t/UM5UZBfdh', -- url
    '', -- files_url
    171, -- impact_score
    '{"solution": "DApp to make dine in reservations/ending time, order food/drinks in advance. Helps businesses plan ahead and customer order are always right"}', -- extra
    'Chris A', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I''m a lover of food and entrepreneur wanting to make the dine in experience even better, while helping businesses succeed', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    40,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Migrate Tron Multi-PVP Game to ADA',  -- title
    'Cardano needs easy-to-learn, multi-player games with ADA as game currency to drive the value of ADA & bring large groups of players together',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'leJpp0PsfTWBQV8QHJ9SVuoqLrF88hlMQuF2DMwAl04=', -- Public Payment Key
    '40000', -- funds
    'http://ideascale.com/t/UM5UZBfdr', -- url
    '', -- files_url
    390, -- impact_score
    '{"solution": "Migrate existing successful Tron DApp to Cardano using KEVM Devnet.\r\n\r\nv1.0 to use ADA and Native Assets as game tokens inside smart contracts"}', -- extra
    'Ragnar Rex', -- proposer name
    '', -- proposer contact
    'https://pc.traps.one/', -- proposer URL
    'Traps has run successfully on Tron for 2yrs  
Daily active users hit 240, top-20 dApp(see attachment)  
User demo: https://youtu.be/op7zkTMhoR8', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    41,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Novellia Gaming Platform and Store',  -- title
    'Blockchain games are difficult for gamers and developers to interact with. There is a need for an ecosystem that simplifies user experience.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'KoyrG9fIjE38QNJ4dJit5mdn4XlaLQY5NwPc8I4Bq/w=', -- Public Payment Key
    '60000', -- funds
    'http://ideascale.com/t/UM5UZBfdv', -- url
    '', -- files_url
    400, -- impact_score
    '{"solution": "Novellia is a community-driven platform that simplifies blockchain integration for developers and users through streamlined user interfaces."}', -- extra
    'rektangular_studios', -- proposer name
    '', -- proposer contact
    'https://rektangularstudios.com/', -- proposer URL
    '3 skilled software developers and a product manager. 20+ years delivering software including AI, blockchain, and gaming.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    42,  -- id
    (SELECT row_id FROM objective WHERE id=6 AND event=4), -- objective
    'Cardano Podcast  Interviews w/Teams',  -- title
    'We don''t know who are the people behind each project on Catalyst. We can only read their and the name of the author.',  -- summary
    (SELECT category FROM objective WHERE id=6 AND event=4), -- category - VITSS Compat ONLY
    'EdWm5rpKOpDJj36u9BmVZ0xK7nN2xpFLT2QlHzJTaK0=', -- Public Payment Key
    '8740', -- funds
    'http://ideascale.com/t/UM5UZBfeg', -- url
    '', -- files_url
    465, -- impact_score
    '{"solution": "Interviews in the Cardano Podcast with the teams behind the interesting projects on Catalyst. This would encourage more participation."}', -- extra
    'Ryan Morrison', -- proposer name
    '', -- proposer contact
    'https://www.youtube.com/channel/UCD-LbBX8c6wgNKggZ43lQrA', -- proposer URL
    'I run the Cardano Podcast. I''ve already conducted interviews with different teams (https://www.youtube.com/channel/UCD-LbBX8c6wgNKggZ43lQrA)', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    43,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Polyswap: Decentralized Exchange',  -- title
    'It should be possible to trade native tokens in a trustless, decentralized manner.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'tswJMnl+VVDbJU+9WTeVFJN0E0C2speBaKZcyOGvSoo=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBfep', -- url
    '', -- files_url
    243, -- impact_score
    '{"solution": "Polyswap will build and maintain an on-chain protocol using Plutus smart contracts for trustless, decentralized trading of native tokens."}', -- extra
    'Daniel Salvadori', -- proposer name
    '', -- proposer contact
    'https://polyswap.io/', -- proposer URL
    'Background in machine learning and computer graphics. Experience in homomorphic encryption and probabilistic programming languages.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    44,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Improving agriculture with Dapps',  -- title
    'The optimization of agricultural techniques is sufficient for self-sustainability.  
The lack of "free" knowledge is the biggest challenge.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    '2QWsXD4dIPLisHcytZmEUrj/9qb+U/vivkW6djLO9ac=', -- Public Payment Key
    '15000', -- funds
    'http://ideascale.com/t/UM5UZBfeq', -- url
    '', -- files_url
    140, -- impact_score
    '{"solution": "We propose a Dapp, where people can access to agricultural techniques,\r\n\r\nand the ability to improve, share and create new ones in Cardano."}', -- extra
    'domebacsi931', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I am a Biophysicist by training (just finishing PhD) and also an entrepreneur.  
Our team is building A.I. applications for industrial usage.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    45,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=4), -- objective
    'One Book a Month',  -- title
    'Write books on a typewriter to represent a stake pool, and publish and distribute the book once the node is saturated.',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=4), -- category - VITSS Compat ONLY
    'Zqor2sf+bMaNYERwjd888P+OMhsByuX3I6jrHWoM5/w=', -- Public Payment Key
    '30000', -- funds
    'http://ideascale.com/t/UM5UZBffN', -- url
    '', -- files_url
    127, -- impact_score
    '{"brief": "Author (and Cardano holder) writes books to support the saturation of stake pool nodes, completing a book for each pool that reaches saturation. Literature is in support of blockchain technology.", "importance": "The typewriting process emulates the purpose of a decentralized network, in that each page is like a block on the chain.", "goal": "Success is saturating a stake pool and releasing literature relating to the human condition.", "metrics": "How quickly a stake pool can be saturated, and whether or not the author can stay on pace. He will."}', -- extra
    'williamofthesun', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    46,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Wrapped Ether - The WETHER token 🌧',  -- title
    'Wrapping Ethereum tokens will add liquidity to the Cardano ecosystem and allow Ethereum users to enjoy scalable Cardano dApps.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    '9psdlWg8I/LZ7d1YllvDen0LbT6ieuCf8MdkF48r9Gk=', -- Public Payment Key
    '30000', -- funds
    'http://ideascale.com/t/UM5UZBffd', -- url
    '', -- files_url
    183, -- impact_score
    '{"solution": "The WETHER station allows users to deposit Ether and mint a wrapped Ether token with 1:1 correspondence, as well as withdraw WETHER to ETHER"}', -- extra
    'Felix', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '6 years enterprise cloud architect, with a specialization in FinTech & security. Currently a Cloud Engineer at Google.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    47,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'ARMing Cardano ',  -- title
    'To scale up a community of low-cost, energy-efficient stake pool operations that will help promote Cardano and our community!',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'Y3uT7y6FgDyZods1cxf1o3qT3JhTEPdw7pbmS+X3EZg=', -- Public Payment Key
    '13943', -- funds
    'http://ideascale.com/t/UM5UZBffo', -- url
    '', -- files_url
    419, -- impact_score
    '{"solution": "Provide education and resources for Raspberry Pi stake pool operators hosted on our ADA.Pi portal to increase decentralization."}', -- extra
    'wael ivie', -- proposer name
    '', -- proposer contact
    'https://github.com/ADA-Pi', -- proposer URL
    'Developer, Stake Pool Operator, Entrepreneur, Educators.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    48,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=4), -- objective
    'Marketing ',  -- title
    'How do we tell as many people as possible about the New World Operating System during the

2021 bull market phase (ending Q4)?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=4), -- category - VITSS Compat ONLY
    'cOtANP8UOQaYjX8mYaejrRf1A88x967j9v8ppgvW8q0=', -- Public Payment Key
    '400000', -- funds
    'http://ideascale.com/t/UM5UZBff5', -- url
    '', -- files_url
    250, -- impact_score
    '{"brief": "A nation-wide (world wide) advertising campaign(s) running in tandem with the Goguen release, a Coinbase listing, and all of the momentum Cardano is building, would fully leverage the inevitable frenzy that will take place in the final parabolic phase of the 2021 crypto bull run.\r\n\r\nIF CARDANO IS TO CHANGE THE WORLD, THE WORLD NEEDS TO KNOW ABOUT CARDANO.\r\n\r\nIt''s up to us to tell them.", "importance": "Informing as many people as possible about Cardano, in an impactful manner, is the most effective way of generating the network effect.", "goal": "A >10% increase in rate of new wallets created and ADA staked, over the course of the marketing campaign(s).\r\n\r\n\\= >ROI than the $ spent.", "metrics": " Cardano related internet searches\r\n\r\n Wallet creation\r\n\r\n ADA staked\r\n\r\n Developer inquiries"}', -- extra
    'Roy Dopson', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    49,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=4), -- objective
    'Partnerships for Global Adoption',  -- title
    'How can Cardano enter in global partnerships with United Nations Development Programme and World Bank Group, to leverage mass adoption?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=4), -- category - VITSS Compat ONLY
    'K5yhWt1aZpyQ4LwSYiDGEyAamlB76ApR5tcRIse8SQw=', -- Public Payment Key
    '30000', -- funds
    'http://ideascale.com/t/UM5UZBfgC', -- url
    '', -- files_url
    270, -- impact_score
    '{"brief": "Partnership protocols aim to support nation building and support sustainable development goals by providing blockchain solutions to local governments and populations. Cardano\u00b4s work would be part of wider development projects undertaking. Proposers to provide a partnership framework model for each of the international bodies in question for Cardano Foundation further implementation.\r\n\r\nhttps://www.undp.org/content/undp/en/home/sustainable-development-goals.html  \r\n\r\nhttps://www.worldbank.org/en/who-we-are", "importance": "Collaboration with International bodies working in the developing world will accelerate mass adoption for Cardano\u00b4s blockchain solutions.", "goal": "Establishment of Local Community Centers in 130 countries backed by Cardano Foundation working in synergy with international bodies", "metrics": "10 proposals put forward for voting\r\n\r\nPartnerships formalized with in 3 months by Cardano Foundation\r\n\r\nLocal Community Centers established with in 6 months"}', -- extra
    'Ronin', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    50,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Vehicle Auctions with Tokenized Bid',  -- title
    'Retail buyers don''t trust dealerships or online auction sites. Current auction sites don''t include transparent, publicly verifiable bidding.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'Nee33TBfgS8qU6yL3wtdW/J8GPW2dppLK7fuOmolMjY=', -- Public Payment Key
    '30000', -- funds
    'http://ideascale.com/t/UM5UZBfgS', -- url
    '', -- files_url
    207, -- impact_score
    '{"solution": "Launch native token used by customers to enter bids with simple credit card on-ramp option. Bids can be easily verified by community."}', -- extra
    'adam', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    ' 4 years building & managing B2B online auction site DoubleClutch.com  
 partnered with Experience Auto Group (largest Ferrari dealer)', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    51,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'PlayerMint: Token System for Gamers',  -- title
    'Currently there is no way for the gaming masses to monetize their gameplay performance without serious time investment or monetary risk.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'fWIBtpftNn5XJ36ajnqAGMzVCaFICWz3KG/2KAt2JdI=', -- Public Payment Key
    '30000', -- funds
    'http://ideascale.com/t/UM5UZBfgi', -- url
    '', -- files_url
    424, -- impact_score
    '{"solution": "PlayerMint will be a free-to-play platform that rewards gamers with fungible and non-fungible tokens based on their gameplay performance."}', -- extra
    'Aidan', -- proposer name
    '', -- proposer contact
    'https://playermint.com/', -- proposer URL
    'Our team of 4 has a combined 45 years of experience in gaming and 10 years in blockchain. We''ve worked with EMURGO, E3, IGN, and IDG/GamePro', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    52,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Circle of Life (DNA)',  -- title
    'People feel alone. People are scared of death. This is due in part to a lack of understanding of the rich human history and evolution.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'D2MIP3/V67mcC2XUQXLaMCQA7Oo/Q65kMNLoq3ImZfg=', -- Public Payment Key
    '80000', -- funds
    'http://ideascale.com/t/UM5UZBfgv', -- url
    '', -- files_url
    150, -- impact_score
    '{"solution": "Make Family Tree Data Integrated"}', -- extra
    'AROCK', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I know how to make things happen.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    53,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=4), -- objective
    'Credit unions and co-operatives ',  -- title
    'How can credit unions extend the higher savings returns from DEFI to less sophisticated and tech savvy investors in developing countries?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=4), -- category - VITSS Compat ONLY
    'QCeqLPVDeCJFvyXxqd3706JlAXgumVgFF3Xv0TlTc9I=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBfhY', -- url
    '', -- files_url
    300, -- impact_score
    '{"brief": "The biggest problem with the financial system today is that the banks and other financial institutions which constitute the majority of this system are profit based. They place a far greater priority on returning profit to their shareholders in preference to providing value to their customers. Credit unions and co-operatives on the other hand are non-profit based and instead focus on returning value to their stakeholders. Cardano is seeking to bring trust back to financial markets via the blockchain. A worthy challenge for Fund 6 is for Cardano to provide an enabling environment for credit unions and co-operatives to flourish. Legal frameworks, related education, thinking, further information etc have already been put in place by the following organisations. https://platform.coop/ https://www.fairshares.coop/ https://youtu.be/qcPUARqRsVM https://youtu.be/2se3c3YHsTc", "importance": "The biggest problem with the financial system today is that the banks which constitute the majority of this system are profit based.", "goal": "Credit unions/co-operatives focusing on returning value to stakeholders are enabled to flourish in competition to profit focused banks.", "metrics": "How many \"existing\" credit unions and co-operatives can we get to \"trial\" Cardano''s enabling environment to \"extend\" the services they provide to their customers?\r\n\r\nHow many \"existing\" credit unions and co-operatives can we get to \"use\" Cardano''s enabling environment to \"extend\" the services they provide to their customers?\r\n\r\nHow many \"new\" credit unions and co-operatives can we get to \"trial\" Cardano''s enabling environment as their operating system?\r\n\r\nHow many \"new\" credit unions and co-operatives can we get to \"use\" Cardano''s enabling environment as their operating system?\r\n\r\nHow many \"existing\" organisations are \"trialling\" Cardano''s enabling environment in order to \"convert\" their organisation into a cooperative?\r\n\r\nHow many \"existing\" organisations have \"used\" Cardano''s enabling environment in order to convert their organisation into a cooperative?"}', -- extra
    'Leonardo koshoni', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    54,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Stake Pool Operator Marketing ',  -- title
    'Small stake pool operators struggle to find delegaters to their stake pool. They rely on incentives like giving to charity to entice people.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    '53pnHYAP32Y9Z3G88isma2Wv8DsuD2YlPyVAZBR57t4=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBfhb', -- url
    '', -- files_url
    164, -- impact_score
    '{"solution": "Creation of a marketing platform that connects the right delegater to the right pool. Dependent upon preferences and obligations."}', -- extra
    'Sean', -- proposer name
    '', -- proposer contact
    'https://www.africanblockchainassociation.co.uk/', -- proposer URL
    'No relevant experience. I am a student who believes in Cardano, who is willing to partner with skilled professionals to make this reality.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    55,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=4), -- objective
    'Nation Building Dapps ',  -- title
    'What core Dapp solutions can be provided for widespread governments adoption in the developing world?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=4), -- category - VITSS Compat ONLY
    'LAWY7snCm5iAYR0K4pVFdL81pWDtGHCz/wH1BsbWp/E=', -- Public Payment Key
    '240000', -- funds
    'http://ideascale.com/t/UM5UZBfhg', -- url
    '', -- files_url
    386, -- impact_score
    '{"brief": "Dapp proposals should cover foundational Registry solutions, namely:  \r\nNational ID & Civil Registry, Property, Medical & Vaccination, Education Census, Revenue Service, Business Incorporations, Elections Voting, Customs & Border Control, Criminal Records, Employment Census, Vehicle Registration, Licensing & Certifications  \r\nProposers to review IOHK/ EMURGO Atala solutions before hand.", "importance": "For the poor nations with no foundations to stand on, Government Dapp solutions will be the building blocks enabling sustainable prosperity", "goal": "Wide spread adoption of Dapps by dozens of governments across developing nations, in collaboration with LCCs and International Partnerships", "metrics": "Minimum 30 proposals covering all core applications in line with challenge brief  \r\n12 winning proposals with ability to deploy Dapps with in 4 months of award  \r\nDapps adoption and implementation during 12 months, sponsored by local governments in 20 countries"}', -- extra
    'Ronin', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    56,  -- id
    (SELECT row_id FROM objective WHERE id=7 AND event=4), -- objective
    'Cardano LCC - Community Outreach DK',  -- title
    'Blockchain & cryptocurrencies generates a rampant increase in public interest, but there is no formal or educational institutions available',  -- summary
    (SELECT category FROM objective WHERE id=7 AND event=4), -- category - VITSS Compat ONLY
    'oDswaoMGLUf6zvcIWW5BY924fp8LHlATg3+O9FzvAh8=', -- Public Payment Key
    '10000', -- funds
    'http://ideascale.com/t/UM5UZBfht', -- url
    '', -- files_url
    333, -- impact_score
    '{"solution": "A LCC would alleviate the rampant increase in demand for an educational institution, and be first movers in an untapped region"}', -- extra
    'Rasmus Moeller Pedersen', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Educational background in marketing management and sales + international business communications.  
Entrepreneur and crypto enthusiast', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    57,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Commodities Dex',  -- title
    'Middle men are more informed than the farmers about the market condition of commodities thus they are more in power to control the prices.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    '1uEyEb+naC4ruxnzNggc8UyqycjvL90yM2hpMH9kcwU=', -- Public Payment Key
    '20000', -- funds
    'http://ideascale.com/t/UM5UZBfhv', -- url
    '', -- files_url
    227, -- impact_score
    '{"solution": "Decentralized exchange for commodities will reduce middle men and allow farmers better information about the market condition."}', -- extra
    'ledifchalang', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'By trade I am an Accountant but self taught web developer for past 1 year now transitioning to Haskell Developer for the past 5 months.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    58,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Cardano Katas',  -- title
    'How to accelerate adoption and development of dApps on Cardano given Haskell is not a mainstream programming language?',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'mvF6pUYJZvgrD7Lhh9uf+o8yyVa6myMUmldskc0UrWc=', -- Public Payment Key
    '10000', -- funds
    'http://ideascale.com/t/UM5UZBfiD', -- url
    '', -- files_url
    220, -- impact_score
    '{"solution": "Prepare a set of code katas for developers to try out one at a time and get trained to code and integrate dApps for real life use cases."}', -- extra
    'reshmhn', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Developer', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    59,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Hub for Javascript Entrepreneurs',  -- title
    'Web developers and entrepreneurs want to build on top of Cardano. Lack of tutorials and boilerplates cause a steep curve to get started.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'fJ0m9OYaKlO7p4axudum1qS4zYQX6EnqB7FdHsjRpEA=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBfiK', -- url
    '', -- files_url
    256, -- impact_score
    '{"solution": "Create a javascript-centric hub of resources for web developers and entrepreneurs looking to build successful dapps/businesses."}', -- extra
    'Collin Glass', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Senior engineer at Trusty (Social Real Estate). Previously, founded an e-sports betting platform and TL @ WaystoCap (African B2B - YC).', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    60,  -- id
    (SELECT row_id FROM objective WHERE id=5 AND event=4), -- objective
    'Free Online Course about Catalyst ',  -- title
    'People outside of the Cardano community don''t know/understand what Catalyst is and how they can apply to get their project funded.',  -- summary
    (SELECT category FROM objective WHERE id=5 AND event=4), -- category - VITSS Compat ONLY
    'cqQiKQ8mXL0uzU0c0KpqQxjLWSbYjN/eRWqKhbi421E=', -- Public Payment Key
    '13280', -- funds
    'http://ideascale.com/t/UM5UZBfiU', -- url
    '', -- files_url
    317, -- impact_score
    '{"solution": "An online course with videos that explains step by step what Catalyst is, how they can submit their proposal, how voting works, etc"}', -- extra
    'Ryan Morrison', -- proposer name
    '', -- proposer contact
    'http://whycardano.org/', -- proposer URL
    '13 years experience in digital marketing. I''ve created the Cardano Podcast and an online academy with courses about digital marketing.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    61,  -- id
    (SELECT row_id FROM objective WHERE id=4 AND event=4), -- objective
    'Diversify Voting Influence',  -- title
    'Voting, like stakepools, requires balanced incentives to encourage a diversity of participants to ensure broad community support.',  -- summary
    (SELECT category FROM objective WHERE id=4 AND event=4), -- category - VITSS Compat ONLY
    '5IbQnPbDCHpiI5QO5TExb7IU5TkRuvGKWnS2iQJvqGU=', -- Public Payment Key
    '14000', -- funds
    'http://ideascale.com/t/UM5UZBfii', -- url
    '', -- files_url
    400, -- impact_score
    '{"solution": "Design and evaluate a variety of voting saturation and aggregation algorithms that balances the influence of small and large stakeholders."}', -- extra
    'Kenric Nelson', -- proposer name
    '', -- proposer contact
    'https://photrek.world', -- proposer URL
    'The Photrek team includes expertise in modeling complex systems, simulating majority vote dynamics, and designing governance policies.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    62,  -- id
    (SELECT row_id FROM objective WHERE id=7 AND event=4), -- objective
    'Poor energy supply/infrastructures',  -- title
    'The problem of providing power supply to ease innovation and provide modern facilities to local communities. Eradicate poverty in local area',  -- summary
    (SELECT category FROM objective WHERE id=7 AND event=4), -- category - VITSS Compat ONLY
    'qxBYZNLDYYminbJzcra043B+Ibr7aaXo4TP54uaICEk=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBfix', -- url
    '', -- files_url
    124, -- impact_score
    '{"solution": "Providing means of reaching out to local communities and establishing companies to promote job opportunities. Providing job training centers"}', -- extra
    'uc_banax', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'The problem of electricity supply in Africa especially Nigeria, has been one of the major challenges they are facing. Innovation is limited.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    63,  -- id
    (SELECT row_id FROM objective WHERE id=4 AND event=4), -- objective
    'Adagov.org (Co-creation) ',  -- title
    'Multiple proposals create a fragmented governance ecosystem with very few coordinating principles. Proposals are unlikely to work together.',  -- summary
    (SELECT category FROM objective WHERE id=4 AND event=4), -- category - VITSS Compat ONLY
    'hrUTvlwA+vRvuICIRBvt4/8NPYqdGPKR7R3A4dNLFDw=', -- Public Payment Key
    '8000', -- funds
    'http://ideascale.com/t/UM5UZBfi2', -- url
    '', -- files_url
    358, -- impact_score
    '{"solution": "If you want to go fast, go alone. If you want to go far, go together.  \r\nLet''s make a team of teams that provide resources and peer-review."}', -- extra
    'Adagov.Org', -- proposer name
    '', -- proposer contact
    'http://www.adagov.org', -- proposer URL
    '1 x Systems Engineering, 1 x Social System Designer (see Voltaire Assistant)', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    64,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'The BookChain Library & Exchange',  -- title
    'Book sales online through retailers gives away 30-70% of the margin to distributors and retailers, while authors and publishers lack data',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'n7YT9Nw7x4GigIsKddIpD+pCuI28rtaT+d1Z69IFWWQ=', -- Public Payment Key
    '25000', -- funds
    'http://ideascale.com/t/UM5UZBfjJ', -- url
    '', -- files_url
    317, -- impact_score
    '{"solution": "The BookChain Library & BookExchange creates incentives for readers to discover and consume content while compensating rights holders direct"}', -- extra
    'Jesse Krieger', -- proposer name
    '', -- proposer contact
    'https://www.LifestyleEntrepreneursPress.com', -- proposer URL
    'Founder & Publisher of Lifestyle Entrepreneurs Press since 2014. Published over 100 books by entrepreneurs. International bestselling author', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    65,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Decentralized Local Marketplace',  -- title
    'Current marketplaces are associated with the trade of illegal items  
Hard to know history of sellers and buyers  
Complicated to prove payment',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'CsJovOuY+RPvYZuM9Jiz5/hMnXzL3qO+1W1OFdOHqmc=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBfjO', -- url
    '', -- files_url
    175, -- impact_score
    '{"solution": "Incentive the community to filter out illegal trades\r\n\r\nSellers and buyers'' history stored on the blockchain\r\n\r\nAbility to prove payment occured"}', -- extra
    'Clement Bisaillon', -- proposer name
    '', -- proposer contact
    'https://github.com/cbisaillon/', -- proposer URL
    'Backend/Frontend development

Solidity smart contract development', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    66,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Synthesis - Hybrid Crowdfunding',  -- title
    'Simple crowdfunding, sales and defi in a single, turnkey platform for individuals, communities, business and enterprises.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'X3/C9NvTuNLhGA6oAN5Qwpwalngv+UgiYYy+vw/nur8=', -- Public Payment Key
    '25000', -- funds
    'http://ideascale.com/t/UM5UZBfjW', -- url
    '', -- files_url
    147, -- impact_score
    '{"solution": "Random number generator raffles providing crowdfunding solutions for : auctions, lotteries, defi, asset sales and financial services."}', -- extra
    'Dunstanlow', -- proposer name
    '', -- proposer contact
    'http://synthesis.finance', -- proposer URL
    '18 years project development, fundraising, marketing, platform and user management.  
PoC -  
https://youtu.be/uhf9uRLRcMY', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    67,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=4), -- objective
    'Developer Courses',  -- title
    'To adopt a training/apprenticeship program for those of us who are looking for a new career',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=4), -- category - VITSS Compat ONLY
    'J3MW0WMvkEsSCZjuxuZyLeN8yduWfcCA+EsqsKbNzCA=', -- Public Payment Key
    '25000', -- funds
    'http://ideascale.com/t/UM5UZBfjf', -- url
    '', -- files_url
    212, -- impact_score
    '{"brief": "The challenge is on me", "importance": "To expand the ability of software companies in using funds to train newbs how to software", "goal": "A highly skilled, built-to-suit full stack developer", "metrics": "Sentiment is measured."}', -- extra
    'Moyk', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    68,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Crowdfunding Platform',  -- title
    'Catalyst offers limited funding. Winning does''t guarantee successful implementation of a project. Proposers need follow-on funding.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'p84xY6ziq56mU/1WK24WicnhT+y3LyizeSqiTlzQwFs=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBfjp', -- url
    '', -- files_url
    253, -- impact_score
    '{"solution": "Innovatio can be a commercial complement of Catalyst organized to help 100s & 1000s of proposal expecting funding by the Cardano treasury."}', -- extra
    'Innovatio', -- proposer name
    '', -- proposer contact
    'https://www.figma.com/proto/4hAZKVfxZaMnbcIFyNisPD/Innovatio?node-id=0%3A1&scaling=min-zoom', -- proposer URL
    'Team of experienced professionals in Startups, Marketing and Tech (AWS, Cyber-security and UIX) with experience in Ts, PostgreSQL and React.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    69,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Rust version of cardano',  -- title
    'Port the current Haskell Cardano implementation to rust, using the similar abstractions.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'BdBjSW7HKuLntawKmWFOzVM1v+yB28NIIglsqunAs1g=', -- Public Payment Key
    '100000', -- funds
    'http://ideascale.com/t/UM5UZBfj2', -- url
    '', -- files_url
    121, -- impact_score
    '{"solution": "Rust have enough type level capabilities for the abstractions used in current Cardano implementation.\r\n\r\n bigger dev community\r\n\r\n runtime"}', -- extra
    'yi.codeplayer', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Have great experiences on both Haskell and Rust.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    70,  -- id
    (SELECT row_id FROM objective WHERE id=6 AND event=4), -- objective
    'Guided Onboarding Experience',  -- title
    'Newbies need more than documentation. When onboarding is self-guided, we create undue friction for incoming talent. Let''s make it personal.',  -- summary
    (SELECT category FROM objective WHERE id=6 AND event=4), -- category - VITSS Compat ONLY
    'KME+0UVPo6f/EHiJFDpI0yWwqPT6G+EmVZYefPpRviE=', -- Public Payment Key
    '15000', -- funds
    'http://ideascale.com/t/UM5UZBfkO', -- url
    '', -- files_url
    463, -- impact_score
    '{"solution": "Deliver a guided onboarding experience to condense ramp-up, triage talent, and motivate DAY 1 action, while creating community connections."}', -- extra
    'Michael McNulty', -- proposer name
    '', -- proposer contact
    'https://app.mural.co/t/teamcatalyst1605/m/teamcatalyst1605/1613753459899/fe645dce5588353ff96b2748221f76859e0ba504', -- proposer URL
    'Co-organized and facilitated first Catalyst Community Retro & Planning

Experience onboarding teams to large scale enterprise initiatives', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    71,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=4), -- objective
    'Staking Pool Workshops',  -- title
    'How can we teach ADA holders how to create and operate their own staking pool?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=4), -- category - VITSS Compat ONLY
    'nAaHWGhGVIKEhVMGRjNbK1Ai0nECjUycuMgU7PM9Hb8=', -- Public Payment Key
    '10000', -- funds
    'http://ideascale.com/t/UM5UZBfkp', -- url
    '', -- files_url
    294, -- impact_score
    '{"brief": "How can running a staking pool be accessible to all community members? Can we create a remote environment (Course or Workshop) in which non-programmers can learn the theory and technical skills to run a Pool?\r\n\r\nThings to consider:\r\n\r\n1) How many days/weeks should this course take? - 2) What theoretical knowledge should operators have? - 3) What technical skills should operators have? - 4) Who will teach this course? - 5) What Platform will be used for this course?", "importance": "This will further decentralize the ecosystem. Providing a wider spectrum of participants to run pools.", "goal": "Give opportunities to non-programmer ADA holders to participate as pool operators.", "metrics": "Apart from bringing new operators to the blockchain, this will ensure that current pool operators actually know what they are doing and have the resources to be successful running their pools.\r\n\r\nNumber of new staking pools in the community.\r\n\r\nNumber of pools being operated by different users."}', -- extra
    'rob', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    72,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Large Example Projects for Devs',  -- title
    'How can we provide a paved path for developers to understand all the technologies that need to come together to build a decent sized dapp?',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    's3MHIRk6WLz3Xdau4BgZGRvgRe1VJZcgvy/KgUrcU8Q=', -- Public Payment Key
    '10000', -- funds
    'http://ideascale.com/t/UM5UZBfks', -- url
    '', -- files_url
    256, -- impact_score
    '{"solution": "Build a reference dapp (for example, a Uniswap v1 clone) from front to back, for people to study, that is well documented and open source."}', -- extra
    'soccer193', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '*   Professional full-stack developer (5 years), in the infosec industry
*   Stake pool operator (Kangaroo Stake Pool)', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    73,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Content Creator Fanbase DEFI DAPP',  -- title
    'Content creators do not have tools to identify their "Most Valuable FANS" and do not have a way to reward fans with crypto or monetize fans',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    '1OR/AVeBi16wem+BIwlQ3FDVp3nZL1twwu9EpWeprfc=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBfkx', -- url
    '', -- files_url
    222, -- impact_score
    '{"solution": "Decentralized finance to let users pay for subscription with their interest automatically. Rank fans by sub and #of people they bring to you"}', -- extra
    'Albert An', -- proposer name
    '', -- proposer contact
    'https://powerfan.io', -- proposer URL
    'Amazon software engineer, Visacard engineer, Samsung engineer. 3rd place Coinbase hackathon, 2nd WCEF https://www.linkedin.com/in/albertahn', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    74,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Howto Cardano',  -- title
    'Lack of good quality entry level/intermediate tutorials. Kids, that never heard about Cardano, now 12 years old, will write the best Dapps.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'QkN9ZmyVmqxl8FKXzRt+cDfr2Up6a/F8Ai2956gnZVI=', -- Public Payment Key
    '100000', -- funds
    'http://ideascale.com/t/UM5UZBfk9', -- url
    '', -- files_url
    158, -- impact_score
    '{"solution": "Cardano was not created in haste and I believe that neither wil mass adoption. The best bet is on easy howto for the 10-12 years old kids."}', -- extra
    'Mihai CATANA', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'wellness instructor, economics studies(not finished),entrepreneur, chef cuisine, tennis instructor, certified masseur,certified nutritionist', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    75,  -- id
    (SELECT row_id FROM objective WHERE id=4 AND event=4), -- objective
    'Voltaire Assistant (Swarm Sessions)',  -- title
    '‌Project Catalyst needs a way to direct decentralized energies supplied by the community into achieving desirable outcomes.',  -- summary
    (SELECT category FROM objective WHERE id=4 AND event=4), -- category - VITSS Compat ONLY
    'TtDQJUW2GQJGIBHALxiiGg4C+2+YJ1P/Hmze3KwMU7g=', -- Public Payment Key
    '7736', -- funds
    'http://ideascale.com/t/UM5UZBflE', -- url
    '', -- files_url
    300, -- impact_score
    '{"solution": "\u200cBuild capacity and systems that facilitate the community in conducting group sessions that focus on creating and achieving goals."}', -- extra
    'Niels Kijf', -- proposer name
    '', -- proposer contact
    'https://miro.com/app/board/o9J_lSgZwMk=/', -- proposer URL
    'Seasoned digital product designer. Developing Voltaire Assistant as distributed decision making software.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    76,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Testing Smart Contracts',  -- title
    'We hear about it all the time. Smart contract exploits that lead to millions in crypto being stolen.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'Xq/LDIINSgTaY953GIIk8AOGwDXzVWkyr+3weAwkTHw=', -- Public Payment Key
    '30000', -- funds
    'http://ideascale.com/t/UM5UZBflI', -- url
    '', -- files_url
    167, -- impact_score
    '{"solution": "A dApp where people can get their smart contract tested by smart contract professionals."}', -- extra
    'Gabriel J', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I am a software developer by profession, an industrial engineer by education. Languages: English, German, French, Mandarin', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    77,  -- id
    (SELECT row_id FROM objective WHERE id=6 AND event=4), -- objective
    'Incentivizing interaction',  -- title
    'Breakthrough innovations are NOT brought about by individuals of similar background. :fearful:',  -- summary
    (SELECT category FROM objective WHERE id=6 AND event=4), -- category - VITSS Compat ONLY
    'QxGlrBjRZOBBeQ4E/G3JobSTA2WCFpTtH77YglmrkP0=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBflR', -- url
    '', -- files_url
    125, -- impact_score
    '{"solution": "Incentivising interaction among people of different backgrounds (the greater the difference, the bigger the reward) brings about innovations"}', -- extra
    'Hiro Goto', -- proposer name
    '', -- proposer contact
    'https://www.inno.go.jp/en/', -- proposer URL
    'My company supports the Japanese government program called innovation.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    78,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Scale Up&Go ',  -- title
    'Decentralise and scale up worker owned gig platform up&go to offer easy adoption in other cities, teams, languages, services and currencies.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'dZ98bv1bJbBqCY6GCm/eb3IvGcivtcF9AgUYlDXIaS4=', -- Public Payment Key
    '18000', -- funds
    'http://ideascale.com/t/UM5UZBflW', -- url
    '', -- files_url
    217, -- impact_score
    '{"solution": "Rebuild an easily adoptable upandgo.coop platform as a Dapp to enable gig workers to take advantage of DeFin, creating fair employment."}', -- extra
    'Carmen Zurl', -- proposer name
    '', -- proposer contact
    'http://www.upandgo.coop', -- proposer URL
    'We are a global consortium of Platform Cooperatives developers, activists, workers and academics scaling show-case working platforms.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    79,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Family/Groups join planning dAPP',  -- title
    'Provide a simple and transparent way for family and groups participate in join projects',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'e+d0FcJ/pY72jOGgocq0jgmB3Xd+OzewjdBsiR+frBU=', -- Public Payment Key
    '80000', -- funds
    'http://ideascale.com/t/UM5UZBflX', -- url
    '', -- files_url
    125, -- impact_score
    '{"solution": "Create a dApp where ppl can share a project so others can pitch in. Can bring families or groups working to a common project."}', -- extra
    'Ricardo ptOWL', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I''m a developer with over 25 years of experience. And I have a big family.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    80,  -- id
    (SELECT row_id FROM objective WHERE id=4 AND event=4), -- objective
    'Infographics - Series 1',  -- title
    'Information about Project Catalyst is difficult to digest for many.',  -- summary
    (SELECT category FROM objective WHERE id=4 AND event=4), -- category - VITSS Compat ONLY
    'IWTdimqs70hzHTVCMTv31HFlE+DOLwhdlxLggaE6tEI=', -- Public Payment Key
    '800', -- funds
    'http://ideascale.com/t/UM5UZBflf', -- url
    '', -- files_url
    459, -- impact_score
    '{"solution": "Create a series of info graphics that visually describe Project Catalyst."}', -- extra
    'Philip Khoo', -- proposer name
    '', -- proposer contact
    'http://philkhoo.com/', -- proposer URL
    'Project Proposer, art and data director: Philip Khoo  

Graphic Designer: Evgeniya Tranevskaya', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    81,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Direct funding: Medusa AdaWallet',  -- title
    'Lack of time to follow roadmap in a good temp because of "enthusiast project" status. But there are a lot of requests from community.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'rp6PWnYvicwD82doNcrdiPe9eBT72VrsOa8VZm58bik=', -- Public Payment Key
    '30000', -- funds
    'http://ideascale.com/t/UM5UZBflw', -- url
    '', -- files_url
    283, -- impact_score
    '{"solution": "Boost development process of the project by providing an ability to leave the main jobs and focus on the Cardano Ecosystem."}', -- extra
    'Fell-x27', -- proposer name
    '', -- proposer contact
    'https://adawallet.io/', -- proposer URL
    'Computer Science degree;

8 years of web-development;

You can check our news and plans here: https://twitter.com/MedusaAdaWallet', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    82,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'AdaStat.net Platform Upgrade',  -- title
    'The upcoming changes in the Cardano will bring a lot of new features, support for which is not implemented in existing blockchain explorers',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'gikyttmGiTth3jPfWUauqw/8wcN4ZYy43Jux4LOFFb4=', -- Public Payment Key
    '20000', -- funds
    'http://ideascale.com/t/UM5UZBfl9', -- url
    '', -- files_url
    333, -- impact_score
    '{"solution": "Our platform will implement support for native tokens, smart contracts, and all other new features in the Cardano blockchain"}', -- extra
    'Dmitry Stashenko', -- proposer name
    '', -- proposer contact
    'https://adastat.net/', -- proposer URL
    'Dmitry Stashenko - creator of adastat.net and https://t.me/AdaStatBot, computer science degree, 12 years of web-development', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    83,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'ADA MakerSpace Hackathon',  -- title
    'As a DEV learning Marlowe and Plutus can be challenging on your own, and as a entrepreneur finding DEVs to work with is also a challenge!',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'PZFE8t11qJ/sybVt7yUrosIMf3KVrlEWou/GeU7XTKI=', -- Public Payment Key
    '30000', -- funds
    'http://ideascale.com/t/UM5UZBfmI', -- url
    '', -- files_url
    186, -- impact_score
    '{"solution": "Host multi region (USA, Ukraine, Africa, ?) in-person and virtual hackathon that provides collaborative space DEVs and Entrepreneurs to meet"}', -- extra
    'Boone Bergsma', -- proposer name
    '', -- proposer contact
    'https://adamaker.space/', -- proposer URL
    'Team members have taken part in over 40 hackathons combined, started a Crypto Café that hosted blockchain events in the past & have partners', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    84,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Native SDKs for iOS and Android',  -- title
    'The lack of native SDKs makes it very hard for app developers to create decentralised apps, which is a significant problem in creating dapps',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'PLnyJzO+gJW5e+8L/S+fwXC9pTDfG2xKY9bK1CQ7Q04=', -- Public Payment Key
    '100000', -- funds
    'http://ideascale.com/t/UM5UZBfmP', -- url
    '', -- files_url
    175, -- impact_score
    '{"solution": "Develop native SDKs for iOS and Android platforms, which will help to develop dapps for these platforms using Cardano blockchain."}', -- extra
    'Adnan', -- proposer name
    '', -- proposer contact
    'https://codexperts.co', -- proposer URL
    'Code Experts Ltd. have a team of experience iOS, Android, React Native and JS Developer. Our developers have more than 11+ yrs of exps.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    85,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Mobile Application for Cardano',  -- title
    'There is no mobile wallet app for the Cardano blockchain for keeping track of and moving ADAs and other tokens and interact with SMRT CNTRt.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'TtUBA1asGmv/EKPlIxDF831BYYRXqYCzBKhF9MSi3A4=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBfmS', -- url
    '', -- files_url
    167, -- impact_score
    '{"solution": "Develop an iOS and Android application that allows users to keep track and move ADAs & native tokens and interact with smart contracts."}', -- extra
    'Adnan', -- proposer name
    '', -- proposer contact
    'https://codexperts.co', -- proposer URL
    'Code Experts Ltd. has a team of talented developers with more than a decade of experience.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    86,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Decentralized Studio on Cardano',  -- title
    'CENTRALIZED Hollywood Studios, Mainstream News & Music can censor, suppress, defame, or blacklist any artist at any time for no reason',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'iV4DtoOkCbNnNnYOZ6VJVkGsSkP198VaSoBjKmgzZMI=', -- Public Payment Key
    '100000', -- funds
    'http://ideascale.com/t/UM5UZBfmf', -- url
    '', -- files_url
    200, -- impact_score
    '{"solution": "Create a DECENTRALIZED studio ecosystem on Cardano to fund & distribute independent artists who desire to benefit people, planet & future"}', -- extra
    'David', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'David A Stone award winning producer, committed to building a decentralized streaming, distribution & funding network for artists on Cardano', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    87,  -- id
    (SELECT row_id FROM objective WHERE id=4 AND event=4), -- objective
    'The Great Filter ❌✅',  -- title
    'Project Catalyst''s form encourages rational ignorance; The more engagement we get, the harder it is to maintain meaningful participation.',  -- summary
    (SELECT category FROM objective WHERE id=4 AND event=4), -- category - VITSS Compat ONLY
    'v4kjg5xTcPFovC+zG7dVhzO3qbyNHNoG93tPrQ0i1/U=', -- Public Payment Key
    '30669', -- funds
    'http://ideascale.com/t/UM5UZBfml', -- url
    '', -- files_url
    419, -- impact_score
    '{"solution": "Empower decision making by filtering proposals with a transparent set of principles, and broadcast the results in a consumer friendly manner"}', -- extra
    'leadtimenull', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'SPOs, Fortune 500 Marketing specialist, Software- & DevOps Engineers, Communication & PR expert  
Languages: EN CN SE DK NO DE RU CZ EE', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    88,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=4), -- objective
    'Cardano Emerging Threat Alarm ',  -- title
    'How can we help stakeholders identify serious emerging systemic threats for the Cardano blockchain before a threat overcomes the system?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=4), -- category - VITSS Compat ONLY
    'u8+7Qyc2YJyzN12WAv5EcH6hOA7HtezpigGBiThfczU=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBfm7', -- url
    '', -- files_url
    460, -- impact_score
    '{"brief": "Blockchain systems are a complex amalgam of technological, economic and social components that rely and are used by many stakeholders from different geographies and backgrounds. These systems are not isolated from the real world and unexpected threats may emerge from the dynamic interaction.  \r\nCardano stakeholders (users, SPOs, DApp developers, partners, Exchanges, governments, companies) need a resilient system that is able to identify and grade developing threats to its own existence.  \r\nCurrently some aspects of the Cardano system are fully centralized, some are becoming decentralized and some are fully decentralized. Stakeholders currently rely on the expertise of Input Output Global (IOG/IOHK) to provide services under contract to the Cardano ecosystem, but the contract is running out. Many stakeholders hope and desire that IOG will submit a new five-year plan for Cardano''s development for funding.  \r\nHowever, even if it materializes as many of us (me included) hope, it will still be good to implement a decentralized threat alert system. Throughout Cardano''s history and the history of other blockchains we have seen how threats can emerge slowly.  \r\nWe often see debates about the dozens of network parameters and their impact on stakeholders, delegators, stakepool operators. There are also governance issues, fee issues, network issues for every blockchain. Often economic issues may apepar, competition from other blockchains, taxation issues regarding staking, the size of the blockchain itself. Future issues might appear as the price of a transaction, spamming the blockchain, attempted Sybil attacks, voting blocks by centralized exchanges that try to influence the network in favor of the interests of the exchanges. Etc\u2026  \r\nIn short, life and social and natural environments are unpredictable. The Cardano network may need to implement a decentralized threat alert system - that does not rely on one actor (IOG/Cardano Foundation/Emergo/you name it).  \r\nA simple threat alert system will alert all stakeholders to emerging issues that are increasingly seen as problems for the Cardano blockchain and as such may generate new funding proposals for dealing with them through funding rounds on Catalyst.  \r\nThink of it as the eyes and ears. In evolutionary biology, it was not by chance that eyes and ears and senses evolved on the front part of animal bodies, as they were the ones that were the first to enter new environments and needed to sense a threat before the threat overcame the system.  \r\nWe need a simple threat alert mechanism to warn us about a not so visible threat that is gaining power and speed. Think of a carbon monoxide alarm. Simple detector for an invisible threat that is building up. Cheap, efficient - and invaluable. That is what this Challenge could lead to.  \r\nStakeholders are the eyes and ears of Cardano. They just need a way of communicating what they are perceiving, so that the feedback can return back to the system. Then it can be dealt with through the existing Catalyst mechanisms and the entire Voltaire structure that is being developed.  \r\nNot all users can detect all threats to be sure. However, the Cardano ecosystem is becoming more diverse in terms of composition, roles, geographic displacement, technical capabilities, personal experience.  \r\nSome threats will be detected by stakepool operators, some will be detected by users, some might be first seen by developers, while other threats will be detected by participants in Project Catalyst.  \r\n  \r\nSome examples of systemic threats that have not been identified or dealt with in other blockchain systems and which may jeopardize their ultimate survival or success are given below and some general threats.  \r\nI''m including a Cardano related threat that has been resolved, number 7 - just to illustrate that our ecosystem has not been immune to problems.\r\n\r\n1.  growing mining centralization in Bitcoin (BTC)\r\n2.  unsustainable gas fees in Ethereum, delays in protocol changes, staking issues (ETH)\r\n3.  alleged non-compliance with securities legislation in US (XRP)\r\n4.  inability to agree on a mechanism to fund ongoing developments or setup a treasury (ETC)\r\n5.  51% attacks and rearrangement of blocks (ETC)\r\n6.  Founder abandonment risk - when founder leaves (EOS)\r\n7.  Unexpected problems inside one of the key ecosystem partners (ADA) - this refers to the period when the Cardano Foundation was headed by its previous chairman and executive director Michael Parsons, which was widely seen by the Cardano community as a dysfunctional period for the Cardano Foundation\r\n8.  Stagnation in evolving the protocol and onboarding new developments (BTC)\r\n9.  Threats from proliferation of blockchain clones - think Bitcoin clones Bitcoin Cash (BCH), Bitcoin SV (BSV), Bitcoin Gold (BTG), Bitcoin Diamond (BCD).\r\n10.  Ongoing scams and crypto phishing attempts are evolving. The more successful Cardano becomes, the more it may be unwillingly associated with scams - in the eyes of users who fall prey to these scammers, leading to negative publicity.\r\n11.  Sybil behavior in evidence. Proliferation of stakepools held by one owner and the consequent increased leverage of such stakepools on the Cardano ecosystem. For example, major centralized exchanges and some pool operators are setting up dozens and dozens of stakepools with the consequent right to have an outsized influence on the future of the Cardano blockchain. \"The higher the leverage of the system, the worse its security (to see this, consider that with leverage above 50, launching a 51% attack requires a mere 1% of the total resources!)\" according to IOHK (ADA)\r\n12.  An unexpected or gradual collapse of one of the key partners in the Cardano ecosystem (IOG/Cardano Foundation/Emurgo) - could happen, although we all hope it won''t\r\n13.  You name it \u2026 it hasn''t occurred yet, but someone in our ecosystem is already noticing it and wants to warn us about it  \r\n     https://iohk.io/en/blog/posts/2020/11/13/the-general-perspective-on-staking-in-cardano/  \r\n    Important note: This is a Fund6 Challenge Setting proposal - for a future challenge in Fund 6. This means I am not personally applying for funding in this challenge! The proposed budget of USD50,000 would go to fund proposals developed by future proposers in Fund 6 who would apply to find solutions to this challenge. I have no proposed solutions nor am I suggesting the best way of addressing this Challenge it will be up to proposers in Fund 6, if this is selected as a future Challenge.", "importance": "Cardano complexity increases with decentralization, native tokens, multi-asset support, smart contracts and new users. Threats will emerge.", "goal": "Stable evolution. Cardano successfully onboards new users, developers, DApps, tokens, SPOs, oracles, companies. Threats are identified", "metrics": "At the end of this challenge, we should be asking ourselves: Did we manage to create a simple mechanism for identifying novel, emerging systemic threats to the Cardano ecosystem? We can''t see the future now, but as events unfold we may be able to use our collective wisdom and senses to detect new dangers for the Cardano blockchain and its usage.\r\n\r\n  \r\n\r\n*   Number of potential threats that have been submitted.\r\n*   Grading of the submitted threats by urgency\r\n*   Number of threats that have been identified as serious and systemic.\r\n*   Grouping of threat sources into categories and types to determine wider danger areas\r\n*   Number of Cardano stakeholders interacting with the threat alert mechanism\r\n*   Number of Catalyst proposals submitted and accepted for funding in subsequent funding rounds to directly address the most pressing identified threats."}', -- extra
    'vladimirp', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    89,  -- id
    (SELECT row_id FROM objective WHERE id=7 AND event=4), -- objective
    'Education/Developer center - Sweden',  -- title
    'There is low adoption and knowledge about blockchain solutions in Sweden.',  -- summary
    (SELECT category FROM objective WHERE id=7 AND event=4), -- category - VITSS Compat ONLY
    '7M0JWa8NvuxiuQn/W7erlxO3HlcBtW8mYLgFl8Lf16U=', -- Public Payment Key
    '2000', -- funds
    'http://ideascale.com/t/UM5UZBfnL', -- url
    '', -- files_url
    281, -- impact_score
    '{"solution": "Educate on a local level and build a local community.  \r\nOffice (Stockholm) at co-working center to build an incubation culture."}', -- extra
    'Johan', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Systems Architect/Developer for 20+ years. Stared to learn about blockchain late 2016 and Cardano mid 2018. Co-organizer of a meetup.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    90,  -- id
    (SELECT row_id FROM objective WHERE id=4 AND event=4), -- objective
    'New Compensation Plan',  -- title
    'Currently only idea creators (Proposers) and some Referrers are being rewarded.

We need to expand compensation to more participants.',  -- summary
    (SELECT category FROM objective WHERE id=4 AND event=4), -- category - VITSS Compat ONLY
    'P3Z/NldI/b6TV+Ndrw/Nxi3QrQiPStTCRRk+qnWMIO4=', -- Public Payment Key
    '20000', -- funds
    'http://ideascale.com/t/UM5UZBfnO', -- url
    '', -- files_url
    175, -- impact_score
    '{"solution": "Create a new compensation plan that reaches out to more participants. Having more people collaborate on an idea and be rewarded accordingly."}', -- extra
    'jeffg', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I have worked in compensation for many years and developing a framework that expands on the initial compensation plan is good for the space.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    91,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Personal Reputation System',  -- title
    'Pandemic isolation create lack of trust among people. It means less opportunities, more poverty and inequality. How can we trust each other?',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'WtrUIbxZuB4TjLjepPGi4pngpuMA06Z/CzGMFdKDAFU=', -- Public Payment Key
    '58000', -- funds
    'http://ideascale.com/t/UM5UZBfnT', -- url
    '', -- files_url
    156, -- impact_score
    '{"solution": "App rewards good actions with tokens given by others. Tokens create a reputation allowing recognize and trust people based on their actions."}', -- extra
    'Rodrigo Frias', -- proposer name
    '', -- proposer contact
    'https://www.karmapoint.app/en', -- proposer URL
    'We are a expert Team! We have +20y in Mgmt, technology, business, Blockchain, Devs. We believe technology can make us better human beings.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    92,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=4), -- objective
    'Propose, build & deliver in 6 weeks',  -- title
    'How can we offer people an opportunity to contribute without having to commit to a large budget while still getting a fair voting chance?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=4), -- category - VITSS Compat ONLY
    'NIj6zk4HzZuWzVj31U9W+MMTW9jyME67uAiolTMs8v8=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBfnU', -- url
    '', -- files_url
    323, -- impact_score
    '{"brief": "The goal of this challenge is to come up with a proposal, build it and deliver all within 6 weeks.  \r\nIdeally the proposals add something to the Catalyst project to help new members or other people who want start making proposals. The work should be shared in such a way that a new proposal can continue from what was delivered in the next funding round. Any code should be shared open source\r\n\r\n  \r\nThe proposals have a max budget of 5k  \r\nThe idea is that all proposals will be of similar size and should have most of their deliverables ready by the time voting starts. This way voting happens within this campaign purely on which teams managed to add real value to the community\r\n\r\n  \r\n*Reasoning for the budget size:*  \r\nProposal budget  \r\nThe goal is to work in a fixed time scope of 6 weeks and to allow new teams to form. These teams would need time to learn and to prove their worth to the stakeholders. With a max of 5k there should be enough budget for a small group to work for several weeks in this fashion, without teams taking on too much. It also leads to a more even valuation within this campaign, so that the value of each proposal is easy to compare for voters (as they are within the same scale).\r\n\r\n  \r\nAny team asking for more should get a very low rating, or be filterd out to keep the campaign fair\r\n\r\n  \r\nCampaign budget  \r\nThe total budget is set at 50k so that at least 10 of these teams can get funded. This encourages people to try and deliver some value, not having to worry about a big fish gobbling up all the funds in the campaign.\r\n\r\n  \r\n(Ideally this would become a recurring campaign to allow for continues development)", "importance": "Large projects come with a lot of risk, both for the executors and the funders. Working in small increments mitigates risks and is flexible", "goal": "New small teams are formed that propose and deliver all within F6, so the deliverables should be done before voting.", "metrics": " Number of teams formed to participate\r\n\r\n Number of teams who delivered before voting\r\n\r\n Number of these proposers continuing on in following funding rounds"}', -- extra
    'SofiH', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    93,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Indie Cardano Node',  -- title
    'We need to demonstrate that the Cardano community is capable of sustaining the development and implementation of the Cardano protocol.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'lhwvfgAJLltTyST3nuogSzS7MQ7TGF2BKuY58NbKbNQ=', -- Public Payment Key
    '1500', -- funds
    'http://ideascale.com/t/UM5UZBfng', -- url
    '', -- files_url
    244, -- impact_score
    '{"solution": "Plan and start the implementation of an independent implementation of the Cardano Protocol."}', -- extra
    'Kiriakos [SPEC]', -- proposer name
    '', -- proposer contact
    'https://kind.software', -- proposer URL
    'Developing software since the 90s. Experienced in web scale applications, jvm, linux, haskell, blockchain (2011) and in building tech teams', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    94,  -- id
    (SELECT row_id FROM objective WHERE id=7 AND event=4), -- objective
    'Training program NOA-NEA Argentina.',  -- title
    'In Argentina, the ignorance of the Cardano ecosystem is complete with more than 2 million potential users of the network.',  -- summary
    (SELECT category FROM objective WHERE id=7 AND event=4), -- category - VITSS Compat ONLY
    'OYR9Q6ZQm1VaS5iB/cHqlYjgsPIFvPKNcTDMGhgJOFo=', -- Public Payment Key
    '45800', -- funds
    'http://ideascale.com/t/UM5UZBfnl', -- url
    '', -- files_url
    275, -- impact_score
    '{"solution": "Implement a training program on the use of Cardano, presenting use cases in the Spanish language to multiplying referents."}', -- extra
    'kyave13', -- proposer name
    '', -- proposer contact
    'https://www.educacionactiva.net/blockchain-cardano/', -- proposer URL
    '10 years of experience as a teacher in the region. Radio producer. Implementation of local training trades.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    95,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=4), -- objective
    'Decentralised Storage Solutions',  -- title
    'How can we create a robust, reliable, secure and affordable decentralised, high-capacity storage network built upon the Cardano blockchain?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=4), -- category - VITSS Compat ONLY
    '6Yu40EJ1Q3Jb8HhtQ2joScNqCHzDCMq4dqvH1qkzu1g=', -- Public Payment Key
    '250000', -- funds
    'http://ideascale.com/t/UM5UZBfno', -- url
    '', -- files_url
    392, -- impact_score
    '{"brief": "The global cloud storage market is projected to grow from USD 49.13 billion in 2019 to USD 297.54 billion by 2027, at a Compound Annual Growth Rate (CAGR) of 25.3% during the forecast period.\r\n\r\nBut perhaps more important than the economic factor, creating a robust, secure and tamper-proof decentralised global storage solution based on blockchain technology would make our data safe, out of the reach of and immune to the whims of large corporations, criminal organisations, or leaders of nation-states.\r\n\r\nSimilar to the projects that already exist like filecoin - https://filecoin.io/ , Holo - https://holo.host/ or the Akash DeCloud - https://akash.network/ storage providers should be incentivised by a token.\r\n\r\nPossibly start with designing and implementing a system that would allow stake pool operators to attach and provide storage to their already existing machines.\r\n\r\nUse Case Example:\r\n\r\nAs a key component in the creation of a trustworthy NFT platform. What use is having the record of ownership on an immutable blockchain, if the work that it points to is no longer accessible? Only the metadata gets stored on the blockchain, if the file it points to is no longer available because the centralised storage it was hosted on is down, the NFT itself loses its value, regardless if it''s a piece of digital art, music, or a land lease.  \r\nPicture how the media would cover the story of an NTF artwork that somebody paid hundreds of thousands of dollars suddenly not being there anymore. After an event such as that, storage permanence would surely become the biggest thing the general public would be concerned about. We can anticipate this by building it into the system from the get-go and setting it up as one of the key differentiators because there''s a possibility it''ll end up being.\r\n\r\nReferences:  \r\nhttps://www.fortunebusinessinsights.com/cloud-storage-market-102773  \r\nhttps://coinmarketcap.com/alexandria/article/what-is-decentralized-storage-a-deep-dive-by-filecoin  \r\nhttps://www.coindesk.com/its-an-nft-boom-do-you-know-where-your-digital-art-lives", "importance": "The world relies more than ever on cloud storage. The need will only increase. Yet a few corporations currently host most of that data.", "goal": "The creation of a decentralised storage solution that makes use of the Cardano blockchain, incentivised by a native token.", "metrics": "Number of participants, peers, in the decentralised storage network.\r\n\r\nNumber of individual users making use of decentralised storage.\r\n\r\nVolume of data stored.\r\n\r\nNumber of organisations and platforms migrated to the decentralised storage solutions.\r\n\r\nNew applications enabled by decentralised storage.\r\n\r\nImpact on the price of storage when compared to centralised storage offerings.\r\n\r\nConsidering this is a complex task that is not likely to be accomplished, and result in a functioning decentralised storage solution, in a 3 to 6 months timeframe we fully expect that proposals would seek funds in multiple rounds of Catalyst for successful implementation. As such the proposals submitted to this challenge would need to present a detailed timeframe and roadmap to implement a working solution."}', -- extra
    'newmindflow', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    96,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'NFT-DAO Framework Collab—continued',  -- title
    'To achieve the de facto NFT platform on Cardano, the Minimum Viable Product of Fund 3 needs to advance to Alpha and Beta testing & release.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'NbnE3P6EQNa4H8jU1cCQOr5M9qSWjkywaNzCSIwJLuA=', -- Public Payment Key
    '39987', -- funds
    'http://ideascale.com/t/UM5UZBfoI', -- url
    '', -- files_url
    267, -- impact_score
    '{"solution": "Complete the Alpha & Beta of the NFT Framework and user interface We will harden the MVP and launch it in Beta while we add new capabilities"}', -- extra
    'rich', -- proposer name
    '', -- proposer contact
    'https://NFT-DAO.org', -- proposer URL
    'The F3 team cover the range of skills and experience needed to build the best possible solution to dominate the NFT space. Details below.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    97,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Myjeeni - Decentralized Healthcare ',  -- title
    'Healthcare research is growing exponentially. But this research isn''t converted into actionable, healthcare protocols at the same rate.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'eddnM5WvbSSurowsKuOFXuDBqcNDCGx3wttnmvDvSn0=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBfoP', -- url
    '', -- files_url
    367, -- impact_score
    '{"solution": "A decentralized, P2P healthcare protocol hub, created by healthcare professionals, using a healthcare DSL and tokenized peer review."}', -- extra
    'suneejnair', -- proposer name
    '', -- proposer contact
    'https://myjeeni.care', -- proposer URL
    'Medical doctor from South Africa.  
Computer programming experience for 15 years. (Haskell, Rust).  
Ycombinator Startup School Graduate..', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    98,  -- id
    (SELECT row_id FROM objective WHERE id=5 AND event=4), -- objective
    'A platform for ordering drugs onlin',  -- title
    'Problem of need for delivering drugs to doorstep of patients especially in light of physical restrictions due to Corona Virus',  -- summary
    (SELECT category FROM objective WHERE id=5 AND event=4), -- category - VITSS Compat ONLY
    'h0zQkWWBls/qOXKptp8cMfluRuHa1GiL1oCtDjNGKYQ=', -- Public Payment Key
    '10000', -- funds
    'http://ideascale.com/t/UM5UZBfoS', -- url
    '', -- files_url
    100, -- impact_score
    '{"solution": "A website where the populace can order for drugs of choice and pay and it is delivered to their doorstep."}', -- extra
    'seyeoyeniyi', -- proposer name
    '', -- proposer contact
    'https://nowpx.com.ng', -- proposer URL
    'Worked in the pharmaceutical sector for about 10 years in logistics, quality assurance, sales and production department.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    99,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'NFT-DAO NFT metadata standards',  -- title
    'NFT use cases have varied metadata needs and schemas. We lack standards for alike NFTs to avoid complex, fragile systems & chaos at scale.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'TtuytVGWk/z0Pm7JFYMAmXVbqrN4zqPlING92/GubJg=', -- Public Payment Key
    '28648', -- funds
    'http://ideascale.com/t/UM5UZBfoW', -- url
    '', -- files_url
    300, -- impact_score
    '{"solution": "We''ll engage industry experts to identify or define standards, work with IOG and industry consortiums to document NFT metadata standards."}', -- extra
    'rich', -- proposer name
    '', -- proposer contact
    'https://github.com/NFT-DAO/NFT-Metadata', -- proposer URL
    'Engineers skilled in standards bodies and marketing team used to collaborating in standard develoment.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    100,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Order Book Based Exchange',  -- title
    'Currently most popular DEXes employ AMM liquidity pools. This lacks the familiarity of an order book based market for spot and derivatives.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'kiLG9gOjj6KvLZIGSk5HMxYhVcCCV2H+zyrpZWsa6kM=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBfoh', -- url
    '', -- files_url
    125, -- impact_score
    '{"solution": "To investigate and implement an order book market to trade ADA/native token pairs, where users can participate in a non-custodial manner."}', -- extra
    'Timothy Wu', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '10+ years building distributed systems

Familiar with event driven methodologies

Proficient in Go, Javascript

Learning Rust, Haskell', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    101,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Free to Play Collectible Game',  -- title
    'In Gacha games, users play to roll for random virtual items. These items are untradeable and only valuable within the game''s ecosystem.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'hx25gh+LQZKNviUKM5M5RvHATCUkWd66AeDudeRsVBI=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBfon', -- url
    '', -- files_url
    222, -- impact_score
    '{"solution": "By producing an innovative game backed by NFTs, users'' in-game items have real-world value. This would also introduce new users to Cardano."}', -- extra
    'jorge gasca', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'The team includes 8 people of diverse skill sets including software development, music, 2D animation, graphic design, and accounting.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    102,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'CardanoPy: 5 min extensible node',  -- title
    'Cardano node setup and ops are overtly difficult. Onboarding and development frictions need to be significantly reduced',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'Ybvw5m94WL3dRMlJc4mOCIZ5oyI6sOcp0GwkIknpeHI=', -- Public Payment Key
    '20000', -- funds
    'http://ideascale.com/t/UM5UZBfo4', -- url
    '', -- files_url
    458, -- impact_score
    '{"solution": "\\# python CLI -> profit!  \r\ncardanopy create --template basic --network testnet files/app  \r\ncardanopy docker run files/app  \r\ncardanopy cli query tip"}', -- extra
    'Bourke Floyd', -- proposer name
    '', -- proposer contact
    'https://github.com/floydcraft/cardano-py', -- proposer URL
    'http://bit.ly/linkedin-bfloyd  
8 yrs soft engineering in mobile gaming  
cardano nodes: rust/haskell testnet w/ dockerized, ci/cd, operations', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    103,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Online Makerspace',  -- title
    'Currently there are no proper way to incentivize Makers, their iterations and ideas.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'qGsyPh4aGX/a7YMHR8kYsl7+reXxiqs2Qq6C/gvi5iM=', -- Public Payment Key
    '25000', -- funds
    'http://ideascale.com/t/UM5UZBfpC', -- url
    '', -- files_url
    179, -- impact_score
    '{"solution": "Through an online makerspace we enable an idea sharing platform around rapid prototyping which puts the Maker in the front seat."}', -- extra
    'Roar Holte', -- proposer name
    '', -- proposer contact
    'https://youblob.com', -- proposer URL
    'The project started back in 2012, where the idea won 1st place in Startup Weekend Stavanger

In 2018, we started aligning it to the industry', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    104,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=4), -- objective
    'Framework for non-tech entrepreneur',  -- title
    'How can we provide a system that allows non-tech entrepreneurs to create PPPs or orther forms of large-scale projects that will use Cardano?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=4), -- category - VITSS Compat ONLY
    'LuCiUgQDGMLkoHydqsxOPQnm8iGgZynf5gPtOgKpDLY=', -- Public Payment Key
    '45000', -- funds
    'http://ideascale.com/t/UM5UZBfpO', -- url
    '', -- files_url
    321, -- impact_score
    '{"brief": "I want to make a difference.\r\n\r\n who do \u0131 talk to within Cardano, is John O''Connor the go-to-man for all private public partnerships?\r\n\r\n what are real world use cases of problems solved with governments?\r\n\r\n what are real world use cases per industry?\r\n\r\n pool of most promising and available (or soon to be available) dApps and what problem they solve?\r\n\r\n Where can \u0131. access the local Cardano community per region and / or per vertical: where do they meet? where do they share ideas? Consolidate in one place most active forums to generate ideas?\r\n\r\n  \r\n\r\nAs a result, with this framework, I w\u0131ll be able to generate or build on an existing idea, find solution partners, be guided by Cardano, and liaise with the government or large enterprise, to bring the idea to life.\r\n\r\nI need this as a businessman. the winning solutions within the Cardano ecosystem right there in one place, user friendly", "importance": "Helping non-tech entrepreneurs understand the opportunities and provide a framework", "goal": "Landing Page:\r\n\r\n Cardano regional or per vertical contact people  \r\n\r\n Example of opportunities (real or potential)\r\n\r\n Process\r\n\r\n match making", "metrics": "\\# of new Business Parnterships and new entrepreneurship activities (projects)\r\n\r\ndiversification across region and capabilities (tech vs non tech, developped vs non-develop countries, 60 year old+ vs youngs, etc)"}', -- extra
    'Kemal Sirin', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    105,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Development Journey Documentation',  -- title
    'Current Plutus documentation is not very user-friendly, lacking real case examples, does not display a journey to Cardano dev eco-system',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'ZsuDc79c4A83fG4jtPtNheSekZO8X/yDoZRaJVxjm6k=', -- Public Payment Key
    '65000', -- funds
    'http://ideascale.com/t/UM5UZBfpo', -- url
    '', -- files_url
    225, -- impact_score
    '{"solution": "Create more user-friendly website, add interesting use cases, accept small community project samples from various fields, community Q/A"}', -- extra
    'Stas', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Computer science degree, +3 years devOps, software engineering, +1 year MLOps, AI (Pytorch), +9 months in blockchain, + some Haskell now', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    106,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Pharma&Biotech - Supply Chain Mngt',  -- title
    'Supply Chain Management in Life Sciences (Pharma, Biotech) is extremely challenging, not cost-efficient with lack of transparency & agility.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'B4XMJsR3oAUb9WCSYQlWauLNaaXQ11Pl5Zg2gyjQ4v4=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBfpp', -- url
    '', -- files_url
    273, -- impact_score
    '{"solution": "Build a transparent Life Sciences Supply chain with enhanced trust, efficiency (payments, supply, storage, etc.) and speed."}', -- extra
    'Mike Barr', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'More than 15 years experience in pharma and biotech in commercial and supply chain management roles (from analyst to global dept. Head)', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    107,  -- id
    (SELECT row_id FROM objective WHERE id=7 AND event=4), -- objective
    'Cardano China Info Hub ',  -- title
    'The Chinese Cardano community has limited access to accurate Cardano-related information which led to low awareness and mass misconception.',  -- summary
    (SELECT category FROM objective WHERE id=7 AND event=4), -- category - VITSS Compat ONLY
    'w4BB4vl+tXP63qsskc+eObC29zUFcjqlKq6wroYijdo=', -- Public Payment Key
    '16700', -- funds
    'http://ideascale.com/t/UM5UZBfps', -- url
    '', -- files_url
    394, -- impact_score
    '{"solution": "Raise awareness and eliminate misconception of Cardano in China by creating accurate, easy-to-digest video and text content in Chinese."}', -- extra
    'Bullish Dumpling', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'SPO, Marketing specialist, Technical Analyst, Technical Content Writer  
Fluent in English and Chinese Mandarin', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    108,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Gimbalabs Building Network Capacity',  -- title
    'To build something truly new, developers need tools, a learning community, and holistic support in elucidating ideas and fostering teams.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'a/Rd7bPNKXIF0DYb71ZtHuGhd4JkWsouwQloctQsDHE=', -- Public Payment Key
    '49350', -- funds
    'http://ideascale.com/t/UM5UZBfp3', -- url
    '', -- files_url
    427, -- impact_score
    '{"solution": "Support experienced developers who are new to blockchain by building tools, creating educational resources, and cultivating project teams."}', -- extra
    'James Dunseith', -- proposer name
    '', -- proposer contact
    'https://gimbalabs.com', -- proposer URL
    '3 Gimbalabs co-founders (Juliane, Roberto, James) + Kyle, an experienced dev/Fund 3 proposer + SofiH, an expert facilitator of agile teams', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    109,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Fiat to ADA stake machine',  -- title
    'Transfer fiat money to ADA staking for people without phone or electricity.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'SgyZecoj0YiwOCjVkb99xWuQllOihVvDh+ZqX8S7Sig=', -- Public Payment Key
    '30000', -- funds
    'http://ideascale.com/t/UM5UZBfp9', -- url
    '', -- files_url
    289, -- impact_score
    '{"solution": "Physical change machine where a customer can deposit fiat and get a QR code ticket in return. Money is staked as ADA."}', -- extra
    'Maarten Menheere', -- proposer name
    '', -- proposer contact
    'https://www.m2tec.nl', -- proposer URL
    'Industrial designer (TU Delft masters). Long experience in building of change machines and POS systems. Some programming experience', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    110,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'DeSci - Decentralised Science',  -- title
    'Academic publication is highly centralised where the value created by authors and peer-reviews is transferred to gate-keeper publishers.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'sPEoET5oFpdrp4FwqciYsNYu3leHMXUwl6SsslVdvzM=', -- Public Payment Key
    '15000', -- funds
    'http://ideascale.com/t/UM5UZBfqc', -- url
    '', -- files_url
    253, -- impact_score
    '{"solution": "Tokenisation of publications with smart contracts in the peer-review process and continued community engagement of the publication."}', -- extra
    'econstable', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Scientific researcher with experience in the peer-review process and an extensive academic network from which to draw test participants.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    111,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'West Africa: Dev Tools & Events',  -- title
    'West Africa has more than 16 countries throughout the region, but there is a lack of community resources to begin developing with Cardano.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'wxJNZdemdG1j7nH5NwnI8OM+zuvh8nbgbtZU13r/00I=', -- Public Payment Key
    '15000', -- funds
    'http://ideascale.com/t/UM5UZBfqk', -- url
    '', -- files_url
    400, -- impact_score
    '{"solution": "Educating developers on Cardano by developing Haskell & Plutus educational material and high-impact virtual workshops, events and hackathons"}', -- extra
    'WADA(West Africa Decentralized Alliance)', -- proposer name
    '', -- proposer contact
    'https://wadaliance.org', -- proposer URL
    'Haskell Devs, Software Devs, Math/Physics Teachers, eLearning Platform. A unique position of having "boots on the ground" local expertise.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    112,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'NFT Based Online Football Manager',  -- title
    'Traditional games lack an important attraction factor: they do not offer users tangible rewards.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'o+MtGOJC1UjTAhIGYClUaL735gatZaYCA3IJuRahm3Q=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBfqq', -- url
    '', -- files_url
    267, -- impact_score
    '{"solution": "Develop an NFT based online football manager game on the Cardano blockchain to attract regular football manager users to the Cardano network"}', -- extra
    'georgianabarbu6', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Software Solution Architecture  

Software Development

Database Development

SPO Ticker: ETR

Functional Analysis

Project Management', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    113,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Real Estate Investment Platform',  -- title
    'Crypto investors don''t have access to real estate investments via crypto funds',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    '2SkyBa0frZsywaLudpJ/6d31Uu9IILMnUWSY7YTmK6U=', -- Public Payment Key
    '10000', -- funds
    'http://ideascale.com/t/UM5UZBfrR', -- url
    '', -- files_url
    273, -- impact_score
    '{"solution": "A platform for investors to purchase a property or join an investment fund that invests in real estate with full transparency"}', -- extra
    'Donny', -- proposer name
    '', -- proposer contact
    'http://estati.ae/', -- proposer URL
    'MBA from Copenhagen Business School, 7 years of IT, product dev and marketing experience, 1.5 years in PropertyTech in Dubai', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    114,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'pruf.io: Media-rich NFTs on Cardano',  -- title
    'Developers and entrepreneurs need a low barrier-to-entry NFT infrastructure to build on or migrate to the Cardano ecosystem.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'FLuKqU6t9PwmOpdemCiHXn4qmbIDTwVAJBwGnzAyquo=', -- Public Payment Key
    '51840', -- funds
    'http://ideascale.com/t/UM5UZBfrS', -- url
    '', -- files_url
    255, -- impact_score
    '{"solution": "Port the PR\u00fcF protocol from Ethereum to Cardano, providing a feature-rich, low-or-no-code NFT onramp for both new and existing projects."}', -- extra
    'Clifford Smyth', -- proposer name
    '', -- proposer contact
    'https://github.com/prufio', -- proposer URL
    'The team has deployed the tested and audited PRüF smart-contract infrastructure and NFT management portal on the ethereum network.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    115,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Ruggedized DeFi',  -- title
    'DeFi apps have exploded in use but have also proved unreliable. Stress testing of DeFi apps for the Cardano environment is required.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    '0uZOj6y6wLLMEeOAw3tubk3WN6+iyrPRqsrrB+MCnq4=', -- Public Payment Key
    '49000', -- funds
    'http://ideascale.com/t/UM5UZBfro', -- url
    '', -- files_url
    179, -- impact_score
    '{"solution": "Tools to assist developers in the verification of DeFi design and agent-based simulation of DeFi prototypes will stress-test proposed apps."}', -- extra
    'Kenric Nelson', -- proposer name
    '', -- proposer contact
    'https://github.com/Photrek/Cardano-Catalyst/tree/main/Ruggedize%20DeFihttps://photrek.world', -- proposer URL
    'The Photrek team provides expertise in the design, analysis, and evaluation of complex open-source software systems.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    116,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Afri-pay',  -- title
    'Problème de transfert d''argent d''un compte bancaire A vers B, car la population ne contrôle pas les transactions effectuées par les banques',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'f/3SFJ2a/+V9iXSOTiOtsDqcP2wmEOhoYq7MAD7GWH8=', -- Public Payment Key
    '9700', -- funds
    'http://ideascale.com/t/UM5UZBfsR', -- url
    '', -- files_url
    218, -- impact_score
    '{"solution": "Bonception d''une plateforme de transfert d''argent bas\u00e9e sur la technologie Blochkchain, d''un compte \u00e0 l''autre sans limite"}', -- extra
    'uptodatedevelopers', -- proposer name
    '', -- proposer contact
    'https://uptodatedevelopers.com', -- proposer URL
    'Nous avons une communauté de développeurs capables de développer des systèmes et nous formons des developpeurs dans la nouvelle technologie', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    117,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    '+Rideshare-CO2 = Ada',  -- title
    'Our planet''s ecosystem is being destroyed by high carbon emissions, worsened by the fact that too many solo drivers are on the road.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'w6agE9g9X+KKlg/HaQQmjxu+QFSvBScSLdiqSKXRkcU=', -- Public Payment Key
    '20000', -- funds
    'http://ideascale.com/t/UM5UZBfsV', -- url
    '', -- files_url
    340, -- impact_score
    '{"solution": "Incentivize ride-sharing by tokenizing and issuing carbon \"credits\" in ADA as a reward to users for CO2 emissions saved on shared trips."}', -- extra
    'Peter Opoku', -- proposer name
    '', -- proposer contact
    'https://www.paalup.com', -- proposer URL
    'PaalUp: 3 fullstack developers one of which also front end/mobile developer  
WADA: Haskell, Elm, & Plutus support and project oversight', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    118,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Make donations transparent',  -- title
    'Companies use donating percentages of their profits as marketing tool. Yet, this is hard and time-consuming to verify for customers.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'eXGP3MD4Ydsj3TWtw/YZLTQAzCs9qt2BKP+5wO8qAgc=', -- Public Payment Key
    '20000', -- funds
    'http://ideascale.com/t/UM5UZBfsg', -- url
    '', -- files_url
    200, -- impact_score
    '{"solution": "A smart-contract to automatically forward x% received at an address to charity organizations. Add explorer function for simple verification."}', -- extra
    'David Grömling', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Software Engineer with 4 years of experience.

Volunteer for a social organization after high-school graduation.

Day to day experience.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    119,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Tokenize West Africa projects',  -- title
    'Interest rates in West Africa are commonly above 20% a year, 87% of innovative projects in Africa do not have access to any funding.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'AvPythdSXKNAnpInnfmX7MB0tXJKDZsjrBjPr84klLw=', -- Public Payment Key
    '200000', -- funds
    'http://ideascale.com/t/UM5UZBfsl', -- url
    '', -- files_url
    236, -- impact_score
    '{"solution": "Create a tokenization ecosystem for financing innovative projects in West Africa.\r\n\r\nhelp technically, structure, and tokezine seed projects."}', -- extra
    'Charles', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Charles lives and works as Blockchain & AI Expert in France and Switzerland, now he is working to help tech startups in West Africa.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    120,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Autographed selfies backed by NFT',  -- title
    'Struggling as a new artist to get your name out? Give your fans an Autographed NFT as your business card and share royalties with your fans.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'aEHg6rLk3F1oNm+PX03698xKHcQc6k7ATqCJM9WHYuY=', -- Public Payment Key
    '55000', -- funds
    'http://ideascale.com/t/UM5UZBfsr', -- url
    '', -- files_url
    213, -- impact_score
    '{"solution": "Graflr authenticates the artists. Artist can sign photos with pre-selected random messages (Rare/Common/Epic). Capture it all with an NFT."}', -- extra
    'Bob Black', -- proposer name
    '', -- proposer contact
    'https://www.graflr.com', -- proposer URL
    'I am a jack of all trades. After 20 years of delivering network cyber security products I wrote a mobile app. Now I want to learn NFTs & ADA', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    121,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Dapps of financial analysis on DEFI',  -- title
    'the future is DEFI and investors need specialized financial analysis before investing.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'atj9k6tpr++sBKsZcNpTh20bnf/J9p+HI34hYvzBPI8=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBfs4', -- url
    '', -- files_url
    100, -- impact_score
    '{"solution": "we will generate financial reports on DEFI to whoever requests it, by a group of securities and crypto professionals"}', -- extra
    'jhonatan smith gutierrez torres', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'The team that makes this proposal is made up of securities professionals with decades of experience and crypto traders and investors.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    122,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Tool to setup and run local testnet',  -- title
    'A tool to setup testnet locally, and tutorials to explore Cardano features in local testnet. Useful for developers to explore Cardano.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'dN4YDd++A4CYORTtUOYxFz5GAS/1RhByyIJZgmPYYD8=', -- Public Payment Key
    '1000', -- funds
    'http://ideascale.com/t/UM5UZBftF', -- url
    '', -- files_url
    206, -- impact_score
    '{"solution": "Write the tool in python (current prototype is in bash)."}', -- extra
    'yi.codeplayer', -- proposer name
    '', -- proposer contact
    'https://github.com/yihuang/cardano-devnet', -- proposer URL
    '10+years software experience with many different languages.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    123,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Mobile Device HW Wallet Integration',  -- title
    'Keeping your keys safe is paramount. Software wallets like Metamask and others work well in the browser, but not as easily on mobile phones.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'RjjWHeBPqd3ng5BZdjENdFLNz9FW5/1lPka5L8+Mfbw=', -- Public Payment Key
    '100000', -- funds
    'http://ideascale.com/t/UM5UZBftR', -- url
    '', -- files_url
    156, -- impact_score
    '{"solution": "A Hardware wallet on smart phones can enable the transacting of ADA between users and DApps."}', -- extra
    'Harris Warren', -- proposer name
    '', -- proposer contact
    'https://developer.samsung.com/blockchain/keystore/overview.html', -- proposer URL
    'Samsung has released a Blockchain Keystore wallet and SDK on all Galaxy 10+ devices. I previously worked at Samsung.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    124,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'CardanoArcade',  -- title
    'The Cardano (ADA) ecosystem needs viable entertainment environment where users can use their ADA in a trusted space/evironment.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'Dt3XLwyTi8Zvt1fbWZNC1whB2IY2jJaOYyZhS/y6x4Q=', -- Public Payment Key
    '200000', -- funds
    'http://ideascale.com/t/UM5UZBfto', -- url
    '', -- files_url
    167, -- impact_score
    '{"solution": "Create a trustless, permission-less, and high-performance e-gaming platform where users can play games like poker, slot machines, and more."}', -- extra
    'Piiggy Bank Labs', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Over 12 years in the entertainment, marketing/advertising/promotion, wholesale & retail sales, professional poker, and entrepreneur spaces.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    125,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Translator ',  -- title
    'With the economy growing, its important more now than ever to have accessible translation devices.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'cKMzxcUU8gjNKOeaqF+T4OIont7d8xgf/55Iz9aiBBw=', -- Public Payment Key
    '10000', -- funds
    'http://ideascale.com/t/UM5UZBftq', -- url
    '', -- files_url
    113, -- impact_score
    '{"solution": "I would produce a coin, that automatically displays everything in your preferred language, whether it be quotes, facts, metrics."}', -- extra
    'nbldwn', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'International celebrities Instagrams

Tv translations

Twitter', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    126,  -- id
    (SELECT row_id FROM objective WHERE id=6 AND event=4), -- objective
    '"They desire a better country."',  -- title
    'Proposers feel left out once their ideas have been rejected, while others further in the Catalyst funnel receive rewards for participating.',  -- summary
    (SELECT category FROM objective WHERE id=6 AND event=4), -- category - VITSS Compat ONLY
    'M9Yh7rlFerccbwCHLIS8R6GPCBVGgr476fCxyloT+Gk=', -- Public Payment Key
    '34000', -- funds
    'http://ideascale.com/t/UM5UZBfty', -- url
    '', -- files_url
    260, -- impact_score
    '{"solution": "Borrowing from traditional heraldry, I propose CF/IOHK issue individual NFT digital assets based on the Canadian Order of Merit system."}', -- extra
    'Citizen Cardano', -- proposer name
    '', -- proposer contact
    'https://en.wikipedia.org/wiki/Orders,_decorations,_and_medals_of_Canada', -- proposer URL
    '\*Digital Marketing (from the E-Commerce space).  
\*Experience with Branding, Trademarking. Relationships with service providers.  
\*Canadian ;).', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    127,  -- id
    (SELECT row_id FROM objective WHERE id=7 AND event=4), -- objective
    'ALDEA, the Latin America DAO',  -- title
    'There is currently no platform or framework for the Cardano Community to discuss, prioritize and solve Latin American issues in Spanish.',  -- summary
    (SELECT category FROM objective WHERE id=7 AND event=4), -- category - VITSS Compat ONLY
    'EVbsuggyCRREzM/+eAUF9331p+v3YHcwYEl2TZ71DL4=', -- Public Payment Key
    '23040', -- funds
    'http://ideascale.com/t/UM5UZBft3', -- url
    '', -- files_url
    353, -- impact_score
    '{"solution": "ALDEA is a DAO that enables people to communicate, self-govern, produce and trade as a decentralized and autonomous Community."}', -- extra
    'Matias Falcone', -- proposer name
    '', -- proposer contact
    'https://aldea-dao.org', -- proposer URL
    'Diverse team of SPOs (FALCO, TOPO, APOLO, RYU, CPOOL), Senior IT Geeks, Entrepreneurs, WyoHackathon Winners & Long-Term Cardano Advocates.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    128,  -- id
    (SELECT row_id FROM objective WHERE id=5 AND event=4), -- objective
    'Viral Media Campaign',  -- title
    'Building a brand reaching 1 billion people will require bold strategy and precise execution when Bitcoin is so far ahead in mindshare',  -- summary
    (SELECT category FROM objective WHERE id=5 AND event=4), -- category - VITSS Compat ONLY
    '8817WunHHNK6sBnxRXD13o14xtbXxXmk+6GvZm14N7U=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBfuA', -- url
    '', -- files_url
    183, -- impact_score
    '{"solution": "Execute a viral media campaign demonstrating the power of Cardano to innovators and investors to spur action to learn more about Cardano"}', -- extra
    'Smithn54', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Technology Leader at top SaaS company', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    129,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Better security using multi-sig',  -- title
    'Storing crypto without any single point of failure by Multi-sig. Aimed for advance users where security is a priority (or institutions).',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'Mh4bCJicIKN9fcV0vi9c8nP5Eh+sPeX/pMGqztkmJrE=', -- Public Payment Key
    '125000', -- funds
    'http://ideascale.com/t/UM5UZBfuz', -- url
    '', -- files_url
    186, -- impact_score
    '{"solution": "Integration to Cobo Vault, an QR code air-gapped hardware wallet with a big touch screen that provides more security and better UX."}', -- extra
    'Lixin', -- proposer name
    '', -- proposer contact
    'https://cobo.com/hardware-wallet/', -- proposer URL
    'I am the Head of Hardware at Cobo. Already participated in the integration with multiple cryptocurrencies.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    130,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=4), -- objective
    'Web-of-Trust Digital Identity',  -- title
    'How can we help indigenous communities build a robust digital identity for members that gives them control over their personal information?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=4), -- category - VITSS Compat ONLY
    '1VTRBD6D9VETkd/DFSZSQGLi0iof/G/3FnTS1JQ7E0Q=', -- Public Payment Key
    '400000', -- funds
    'http://ideascale.com/t/UM5UZBfu3', -- url
    '', -- files_url
    325, -- impact_score
    '{"brief": "Making it easier for grass-roots membership organisations and indigenous groups to onboard members and help them create a digital identity that is under their control will lead to increased adoption of Cardano-based DApps and be the basis for developing an economic identity for these NGOs, social organisations and indigenous cultures.  \r\nGiving community members the tools needed to educate, help and onboard others in their community to set up digital identities easily. Individuals are able to get a Cardano (Atala Prism) based digital identity set-up and initial non-government credentials verified, such as belonging to a savings group, being a member of a tribe.\r\n\r\nCharities, Governments agencies, employers or tribes can easily establish and produce individual claims and issue them to individuals.\r\n\r\nIndigenous organisations in particular such as tribal trusts and companies are able to integrate with Cardano''s DiD based framework and enable better communication with their members.\r\n\r\nAn outline of the needs and community lead approach for designing membership registers for M\u0101ori organisations in Aotearoa (NZ) can be found in the document A R\u014dp\u016b T\u012bkanga M\u0101ori Membership Database Toolkit (see attached PDF https://cardano.ideascale.com/a/idea/341410/35461/download ). What would such a toolkit be if incorporated Self-Sovereign Identity perspectives?\r\n\r\nConsider for instance developing indigenous trust frameworks that focus on the sharing and reuse of data. For instance in conjunction with Te Mana Raraunga (M\u0101ori Data Sovereignty) https://www.temanararaunga.maori.nz/\r\n\r\nRelated Information\r\n\r\nA Closely related challenge proposal from Fund3 is Atala PRISM DID Mass-Scale Adoption https://cardano.ideascale.com/a/dtd/Atala-PRISM-DID-Mass-Scale-Adoption/334524-48088\r\n\r\nIOG''s Atala Prism prototype: https://atalaprism.io\r\n\r\nW3C DID Standards: https://www.w3.org/TR/did-core/\r\n\r\nW3C DID Use Cases: https://www.w3.org/TR/did-use-cases/\r\n\r\nDecentralized Identity Foundation (DIF) https://identity.foundation/\r\n\r\nRebooting Web-of-Trust: https://www.weboftrust.info/\r\n\r\nA REMINDER: This is a Fund6 Challenge Setting proposal. It is not a proposal to build something, but a proposal for a future challenge in another fund where people submit proposals to complete the challenge.", "importance": "Indigenous communities struggle to retain their culture. Digital Identity can strengthen collective identity and economic wellbeing.", "goal": "Community members have the tools needed to educate, help and onboard others in their community to set up digital identities easily.", "metrics": "To assess the ROI of this challenge we will ask ourselves: Did we get engagement with indigenous and grass-roots communities and were they issuing members digital identity credentials by the end of the fund time horizon?\r\n\r\n*   How many proposals directly engage with indigenous communities and organisations?\r\n*   What number of useful training resources and guides are developed for different communities in their language?\r\n*   What was the number of integrations with indigenous organisation membership systems?\r\n*   Did any Tribal, NGOs or Charities start issuing or relying on the created credentials?\r\n*   How many new and novel use-cases for indigenous digital identity were developed.\r\n*   Was there a sense of community and conversations on how to improve the economic identity of indigenous communities with Cardano?\r\n*   Where any credential standards developed that captured the needs of a particular group such as tribal membership.\r\n*   Where any trust-frameworks (governance protocols) developed for particular member organisations."}', -- extra
    'Robert O''Brien', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    131,  -- id
    (SELECT row_id FROM objective WHERE id=7 AND event=4), -- objective
    'Adriatic EU Community Center',  -- title
    'Lack of a physical place for Catalyst Community to meet, socialize and represent Cardano network.',  -- summary
    (SELECT category FROM objective WHERE id=7 AND event=4), -- category - VITSS Compat ONLY
    'yKnz/knTjqrSFK9+2kM7nINR3+yx/N35uWVSdo5EA/4=', -- Public Payment Key
    '12000', -- funds
    'http://ideascale.com/t/UM5UZBfvB', -- url
    '', -- files_url
    352, -- impact_score
    '{"solution": "Providing Cardano education-promotional service center (CEPS), in business complex center in central Istria-Croatia."}', -- extra
    'F', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Team of a 20 years experience product owner and CEO of an IT company which is owner of the site, 7 developers, engineer', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    132,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'TipADA - Decentralized $ADA Tipping',  -- title
    'There is a great demand for showing appreciation of content and ideas by tipping $ADA to a creator. Currently no solution is available.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'TuIH8SDS8w9RLAlVbl1zUqmGORauyR2geIv+9f+JrRs=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBfvE', -- url
    '', -- files_url
    367, -- impact_score
    '{"solution": "A possible solution is to create a browser extension for all major platform to facilitate tipping $ADA tokens on multiple social platforms."}', -- extra
    'Clark Alesna', -- proposer name
    '', -- proposer contact
    'https://www.codementor.io/@mercurial', -- proposer URL
    'I am Clark Alesna, the operator of ADAPH stake pool.  
I have been a Software Engineer for 10+ years  
https://www.linkedin.com/in/clarkalesna/', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    133,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Visual Studio Code Market Plugin',  -- title
    'There are no plugins within for Microsoft Visual Studio Code to build Cardano Smart Contracts on vb/c# .NET. VSCODE is used by 1000s of devs',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'AXMfQ+ENX248aBqLNPD2ChqiFlsAkUx3U6K808MFJwg=', -- Public Payment Key
    '23450', -- funds
    'http://ideascale.com/t/UM5UZBfvT', -- url
    '', -- files_url
    382, -- impact_score
    '{"solution": "FREE plugin will simplify Cardano Smart Contract creations with a project template and instructions to attract the wider OCEAN developers"}', -- extra
    'Rob Greig', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I have been developing on Visual Studio for over 20 years and have built many plugins and extensions for the development tool.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    134,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'HW wallet passphrase recovery tool',  -- title
    'Currently there is no public tool supporting Cardano allowing BIP39 passphrase to be guessed/tested. Plus none using GPU acceleration.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'uqoGN9jIddosL036h3V39hIB00uhLj8TT7BjxHf2wEM=', -- Public Payment Key
    '2500', -- funds
    'http://ideascale.com/t/UM5UZBfvX', -- url
    '', -- files_url
    350, -- impact_score
    '{"solution": "Add Cardano (ADA) support and GPU acceleration support to widely used and publicly known tool named btcrecover."}', -- extra
    'Dusan - lunarpool.io', -- proposer name
    '', -- proposer contact
    'https://github.com/3rdIteration/btcrecover', -- proposer URL
    'As a SPO we were asked to help an ADA holder with this issue and there are more to come for sure. There''s no such tool at the moment.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    135,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'JavaScript SDK for Blockfrost API',  -- title
    'JavaScript developers are missing tools to fully enjoy Blockfrost.io, a service that provides free and public Cardano API to developer.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'B0ZvLFPPlk9E2MkVDqrtgES5G7V1dwdfQE+iBSZZH8c=', -- Public Payment Key
    '9500', -- funds
    'http://ideascale.com/t/UM5UZBfva', -- url
    '', -- files_url
    429, -- impact_score
    '{"solution": "We want to build an open-source SDK (Software Development Kit - a set of tools, libraries and documentation) for JavaScript developers."}', -- extra
    'Five Binaries', -- proposer name
    '', -- proposer contact
    'https://blockfrost.io/', -- proposer URL
    'We are the creators of Blockfrost.io, creating services and tools lowering barriers to entry for developers.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    136,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Python SDK for Blockfrost API',  -- title
    'Python developers are missing tools to fully enjoy Blockfrost.io, a service that provides free and public Cardano API to developers.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'Vlf6LIs1h0xttcNHGL/PFhd4Lfd5jxBV0KxB7KGVgc0=', -- Public Payment Key
    '9500', -- funds
    'http://ideascale.com/t/UM5UZBfvb', -- url
    '', -- files_url
    439, -- impact_score
    '{"solution": "We want to build an open-source SDK (Software Development Kit - a set of tools, libraries and documentation) for Python developers."}', -- extra
    'Five Binaries', -- proposer name
    '', -- proposer contact
    'https://blockfrost.io/', -- proposer URL
    'We are the creators of Blockfrost.io, creating services and tools lowering barriers to entry for developers.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    137,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Golang SDK for Blockfrost API',  -- title
    'Go language developers are missing tools to fully enjoy Blockfrost.io, a service that provides free and public Cardano API to developers.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'xL0n+SIyy+OtDqqDx2xzAdBjSLPUhatI/v58v+t/36c=', -- Public Payment Key
    '7000', -- funds
    'http://ideascale.com/t/UM5UZBfvd', -- url
    '', -- files_url
    427, -- impact_score
    '{"solution": "We want to build an open-source SDK (Software Development Kit - a set of tools, libraries and documentation) for Go developers."}', -- extra
    'Five Binaries', -- proposer name
    '', -- proposer contact
    'https://blockfrost.io/', -- proposer URL
    'We are the creators of Blockfrost.io, creating services and tools lowering barriers to entry for developers.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    138,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Haskell SDK for Blockfrost API',  -- title
    'Haskell developers are missing tools to fully enjoy Blockfrost.io, a service that provides free and public Cardano API to developers.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'eSMT5LsSKH5Zn1eaeUbxa/bdIYRYokUQtZ7EK/j6jmU=', -- Public Payment Key
    '8000', -- funds
    'http://ideascale.com/t/UM5UZBfve', -- url
    '', -- files_url
    450, -- impact_score
    '{"solution": "We want to build an open-source SDK (Software Development Kit - a set of tools, libraries and documentation) for Haskell developers."}', -- extra
    'Five Binaries', -- proposer name
    '', -- proposer contact
    'https://blockfrost.io/', -- proposer URL
    'We are the creators of Blockfrost.io, creating services and tools lowering barriers to entry for developers.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    139,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Rust SDK for Blockfrost API',  -- title
    'Rust developers are missing tools to fully enjoy Blockfrost.io, a service that provides free and public Cardano API to developers.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    '13ZgekorrI+fbOucjjWwBS6wroAF4wSylTa+zYDLPQI=', -- Public Payment Key
    '7000', -- funds
    'http://ideascale.com/t/UM5UZBfvg', -- url
    '', -- files_url
    467, -- impact_score
    '{"solution": "We want to build an open-source SDK (Software Development Kit - a set of tools, libraries and documentation) for Rust developers."}', -- extra
    'Five Binaries', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'We are the creators of Blockfrost.io, creating services and tools lowering barriers to entry for developers.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    140,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Swift SDK for Blockfrost API',  -- title
    'Swift developers are missing tools to fully enjoy Blockfrost.io, a service that provides free and public Cardano API to developers.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'pfqjUb4pl2eUXB92ww6N6CmUHbW+4aFtd5Uf2J4k4OA=', -- Public Payment Key
    '9500', -- funds
    'http://ideascale.com/t/UM5UZBfvj', -- url
    '', -- files_url
    440, -- impact_score
    '{"solution": "We want to build an open-source SDK (Software Development Kit - a set of tools, libraries and documentation) for Swift developers."}', -- extra
    'Five Binaries', -- proposer name
    '', -- proposer contact
    'https://blockfrost.io/', -- proposer URL
    'We are the creators of Blockfrost.io, creating services and tools lowering barriers to entry for developers.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    141,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'PHP SDK for Blockfrost API',  -- title
    'PHP developers are missing tools to fully enjoy Blockfrost.io, a service that provides free and public Cardano API to developers.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    '/oYJlHa5skCuA4Z2XI8xdYHVnkmLZ4n19MaggmAeqD8=', -- Public Payment Key
    '7000', -- funds
    'http://ideascale.com/t/UM5UZBfvk', -- url
    '', -- files_url
    433, -- impact_score
    '{"solution": "We want to build an open-source SDK (Software Development Kit - a set of tools, libraries and documentation) for PHP developers."}', -- extra
    'Five Binaries', -- proposer name
    '', -- proposer contact
    'https://blockfrost.io/', -- proposer URL
    'We are the creators of Blockfrost.io, creating services and tools lowering barriers to entry for developers.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    142,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Scala SDK for Blockfrost API',  -- title
    'Scala developers are missing tools to fully enjoy Blockfrost.io, a service that provides free and public Cardano API to developers.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'm0nrTVaW+Rk8SZ4GgffnQOoI6kFFfhf8eZsJvyc2ZX0=', -- Public Payment Key
    '9500', -- funds
    'http://ideascale.com/t/UM5UZBfvn', -- url
    '', -- files_url
    427, -- impact_score
    '{"solution": "We want to build an open-source SDK (Software Development Kit - a set of tools, libraries and documentation) for Scala developers."}', -- extra
    'Five Binaries', -- proposer name
    '', -- proposer contact
    'https://blockfrost.io/', -- proposer URL
    'We are the creators of Blockfrost.io, creating services and tools lowering barriers to entry for developers.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    143,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Ruby SDK for Blockfrost API',  -- title
    'Ruby developers are missing tools to fully enjoy Blockfrost.io, a service that provides free and public Cardano API to developers.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'Y3MGkdpkIz9EMv9ojcJQTs7OnkJT0MoftRBcf7Dh3dY=', -- Public Payment Key
    '7000', -- funds
    'http://ideascale.com/t/UM5UZBfvo', -- url
    '', -- files_url
    442, -- impact_score
    '{"solution": "We want to build an open-source SDK (Software Development Kit - a set of tools, libraries and documentation) for Ruby developers."}', -- extra
    'Five Binaries', -- proposer name
    '', -- proposer contact
    'https://blockfrost.io/', -- proposer URL
    'We are the creators of Blockfrost.io, creating services and tools lowering barriers to entry for developers.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    144,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Java SDK for Blockfrost API',  -- title
    'Java developers are missing tools to fully enjoy Blockfrost.io, a service that provides free and public Cardano API to developers.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'PrRuJCFZ0uhqHbtwaYz7c6Cp6kKzzH0KVVSFjycB3IQ=', -- Public Payment Key
    '8000', -- funds
    'http://ideascale.com/t/UM5UZBfvp', -- url
    '', -- files_url
    415, -- impact_score
    '{"solution": "We want to build an open-source SDK (Software Development Kit - a set of tools, libraries and documentation) for Java developers."}', -- extra
    'Five Binaries', -- proposer name
    '', -- proposer contact
    'https://blockfrost.io/', -- proposer URL
    'We are the creators of Blockfrost.io, creating services and tools lowering barriers to entry for developers.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    145,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Arduino SDK for Blockfrost API',  -- title
    'Arduino developers are missing tools to fully enjoy Blockfrost.io, a service that provides free and public Cardano API to developers.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'DUmQJPx5quTf6MfJFuD2kD1DzZOQxl+pjl5yVUxbXXA=', -- Public Payment Key
    '9000', -- funds
    'http://ideascale.com/t/UM5UZBfvq', -- url
    '', -- files_url
    450, -- impact_score
    '{"solution": "We want to build an open-source SDK (Software Development Kit - a set of tools, libraries and documentation) for Arduino developers."}', -- extra
    'Five Binaries', -- proposer name
    '', -- proposer contact
    'https://blockfrost.io/', -- proposer URL
    'We are the creators of Blockfrost.io, creating services and tools lowering barriers to entry for developers.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    146,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Elixir SDK for Blockfrost API',  -- title
    'Elixir developers are missing tools to fully enjoy Blockfrost.io, a service that provides free and public Cardano API to developers.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'jBW0lyPxGkfB1Nt6W8U/BNqY/d8nZtZOs5AB2KNc4PI=', -- Public Payment Key
    '7000', -- funds
    'http://ideascale.com/t/UM5UZBfvr', -- url
    '', -- files_url
    442, -- impact_score
    '{"solution": "We want to build an open-source SDK (Software Development Kit - a set of tools, libraries and documentation) for Elixir developers."}', -- extra
    'Five Binaries', -- proposer name
    '', -- proposer contact
    'https://blockfrost.io/', -- proposer URL
    'We are the creators of Blockfrost.io, creating services and tools lowering barriers to entry for developers.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    147,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    '.NET SDK for Blockfrost API',  -- title
    '.NET developers are missing tools to fully enjoy Blockfrost.io, a service that provides free and public Cardano API to developers.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'A/zWTZf1QFmsDq752hVXqsEgSbM6ISCknG6oEPz5xUI=', -- Public Payment Key
    '8300', -- funds
    'http://ideascale.com/t/UM5UZBfvt', -- url
    '', -- files_url
    443, -- impact_score
    '{"solution": "We want to build an open-source SDK (Software Development Kit - a set of tools, libraries and documentation) for .NET developers."}', -- extra
    'Five Binaries', -- proposer name
    '', -- proposer contact
    'https://blockfrost.io/', -- proposer URL
    'We are the creators of Blockfrost.io, creating services and tools lowering barriers to entry for developers.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    148,  -- id
    (SELECT row_id FROM objective WHERE id=7 AND event=4), -- objective
    ' Global sustainable transport',  -- title
    'The continuous advance of society causes every day more climate problems, the planet dies and Cardano can help save it :grinning:',  -- summary
    (SELECT category FROM objective WHERE id=7 AND event=4), -- category - VITSS Compat ONLY
    'PNdTW21NdFMrmmg5pH03/sgvjPivuz+AeFZEp7aBEO0=', -- Public Payment Key
    '25000', -- funds
    'http://ideascale.com/t/UM5UZBfvv', -- url
    '', -- files_url
    117, -- impact_score
    '{"solution": "A globally unified sustainable transport system thanks to a payment system with a common token which allows you to always use it"}', -- extra
    'Arturo Pozuelo Calvet', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'The user can earn rewards in tokens for using public transport and also hold the tokens in their own wallet.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    149,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Yoroi ⇄ Blockfrost bridge',  -- title
    'Yoroi developement often requires full stack Cardano node + DB Sync, because Yoroi implements only a basic subset of available endpoints.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'sstLfcbmTE2ZnC1yGjb+Zk0IH+o1aUApO/Pd5RI4TVw=', -- Public Payment Key
    '6500', -- funds
    'http://ideascale.com/t/UM5UZBfvx', -- url
    '', -- files_url
    450, -- impact_score
    '{"solution": "Build a bridge between Yoroi and Blockfrost.io, so that developers can use extended endpoint set to develop advanced features more easily."}', -- extra
    'Five Binaries', -- proposer name
    '', -- proposer contact
    'https://blockfrost.io/', -- proposer URL
    'We are the creators of Blockfrost.io, creating services and tools lowering barriers to entry for developers.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    150,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Token faucet',  -- title
    'It is expensive to faucet out or airdrop tokens, as you need to include more than 1 ADA for each transaction.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'b95e1aG+OiOm6ZVOsRlXgGjbY6KvwtiK7R7HSY9YS+Y=', -- Public Payment Key
    '9500', -- funds
    'http://ideascale.com/t/UM5UZBfvy', -- url
    '', -- files_url
    247, -- impact_score
    '{"solution": "We would like to develop a server side component to facilitate the airdropping or faucet of native tokens on Cardano"}', -- extra
    'Five Binaries', -- proposer name
    '', -- proposer contact
    'https://fivebinaries.com/', -- proposer URL
    'We minted the first Cardano native token in history: nutcoin. We have implemented the native asset support to Blockfrost.io.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    151,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'API for decentralized application',  -- title
    'The need to have rapid development tools already tested is observed',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'OfrroYCwEL9MUAdVVbgaMQh5sysXhO3gWezZmgZPF9Q=', -- Public Payment Key
    '1500', -- funds
    'http://ideascale.com/t/UM5UZBfv2', -- url
    '', -- files_url
    117, -- impact_score
    '{"solution": "in order to shorten development times and achieve effective implementation with APIs developed for that purpose."}', -- extra
    'Hugo Ojeda', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'software engineer', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    152,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Multiplayer Game using Cardano',  -- title
    'Making Cardano mainstream. Training young developers.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    '3tmvM1j60AbmswphPtuXtp3cAUbybbn/A3XPS6zsWT8=', -- Public Payment Key
    '100000', -- funds
    'http://ideascale.com/t/UM5UZBfv4', -- url
    '', -- files_url
    200, -- impact_score
    '{"solution": "In my multiplayer online game, I want to implement Cardano for asset management, contracts and payments (internal and external)."}', -- extra
    'slarti', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Over 25 years experience in programming, financial and computer science studies.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    153,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Making Cardano the go to for games',  -- title
    'How can game developers, especially indie developers, access and easily implement a payment system, circumventing the big payment providers?',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'urtP0mIHaq91hQ+HKzMt2J9sjuPR67rQf6v2iMXzaIY=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBfwH', -- url
    '', -- files_url
    109, -- impact_score
    '{"solution": "Providing an easy toolset, accessible through asset store like under Unity development platform and providing the payment service."}', -- extra
    'slarti', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Over 25 years experience in programming, financial and computer science studies.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    154,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Making Cardano the go to for games',  -- title
    'How can game developers, especially indie developers, access and easily implement a payment system, circumventing the big payment providers?',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'wsDdwdzC6xhOdtfCsJ4xG2vw9SCL2W8FmuI9TNvQZfU=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBfwJ', -- url
    '', -- files_url
    167, -- impact_score
    '{"solution": "Providing an easy toolset, accessible through asset store like under Unity development platform and providing the payment service."}', -- extra
    'slarti', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Over 25 years experience in programming, financial and computer science studies.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    155,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Blockchain in Education',  -- title
    'Generally, significant higher education blockchain use cases are, for instance, record-keeping and performance optimization.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'LHAY+yJWyznTUNO49zKVMcag6Uyhm6FtnjwGtzwi8Oc=', -- Public Payment Key
    '2000', -- funds
    'http://ideascale.com/t/UM5UZBfwM', -- url
    '', -- files_url
    220, -- impact_score
    '{"solution": "This new educational platform uses blockchain smart contracts to automate administrative tasks, protect and secure faculty and students."}', -- extra
    'Hugo Ojeda', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'software engineer', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    156,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    ' Global sustainable transport',  -- title
    'The continuous advance of society causes every day more climate problems, the planet dies and Cardano can help save it :grinning:',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'RxfXbFEXo61qzKVnWw74kGMYZ2FhvxCKcKBBSUiIJL0=', -- Public Payment Key
    '25000', -- funds
    'http://ideascale.com/t/UM5UZBfwQ', -- url
    '', -- files_url
    150, -- impact_score
    '{"solution": "A globally unified sustainable transport system thanks to a payment system with a common token which allows you to always use it"}', -- extra
    'Arturo Pozuelo Calvet', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'The user can earn rewards in tokens for using public transport and also hold the tokens in their own wallet.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    157,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Dapp to control/monetize your data ',  -- title
    '$300B surveillance capitalism market broke the customer relationship. People should control and benefit from the use of their personal data',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    '4S8PJnHrASGUDZ1CLdXfjMMgSFFUeGbnwrGfCBQ8RJY=', -- Public Payment Key
    '23200', -- funds
    'http://ideascale.com/t/UM5UZBfwX', -- url
    '', -- files_url
    338, -- impact_score
    '{"solution": "Dapp and permission-based customer platform, using personal data-licensing smart contracts, which pays people for their attention."}', -- extra
    'Michiel Van Roey ', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Privacy/Tech lawyer with 10Y XP in (project relevant) legal issues - consumer privacy; digital marketing; data rights & virtual currencies.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    158,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'ADAppTO',  -- title
    'Its too complicated to cash in and cash out ADA through coinbase, binance and similar websites and you always lose money on fees.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'lty8uYB66MPm1St48Zl7ySg4s5nJRiSwTSKFlh1Ipow=', -- Public Payment Key
    '10000', -- funds
    'http://ideascale.com/t/UM5UZBfwu', -- url
    '', -- files_url
    100, -- impact_score
    '{"solution": "\"ADAppTO\"-an app where one could simply convert their fiat currency into ADA and vice versa. then send ada to ada wallets or fiat to bank."}', -- extra
    'Marin Tironi', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I have no experience in app developing whatsoever, but thiught I might throw the idea out there.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    159,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Kotlin SDK for Blockfrost API',  -- title
    'Kotlin developers are missing tools to fully enjoy Blockfrost.io, a service that provides free and public Cardano API to developers.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'PJXfnqjs2jkuX/WuMpxyhKW6MKRiYkIyDW1MeJH3a8I=', -- Public Payment Key
    '9000', -- funds
    'http://ideascale.com/t/UM5UZBfw6', -- url
    '', -- files_url
    427, -- impact_score
    '{"solution": "We want to build an open-source SDK (Software Development Kit - a set of tools, libraries and documentation) for Kotlin developers."}', -- extra
    'Five Binaries', -- proposer name
    '', -- proposer contact
    'https://blockfrost.io/', -- proposer URL
    'We are the creators of Blockfrost.io, creating services and tools lowering barriers to entry for developers.blockf', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    160,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Decentralized exchange - VyFinance',  -- title
    'There''s a need for dex on every blockchain, and cardano seems to fit the best with what a dex can offer.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    '/W5T5AqC8HOJH1QspWEHi0n098DPdwp23Z8DM+sYh1s=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBfw8', -- url
    '', -- files_url
    100, -- impact_score
    '{"solution": "We are working on creating pooling, yield farming, swapping, lending and borrowing with our own crypto token used for governance"}', -- extra
    'cardano.vyfinance', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Team started this travel working in solidity (https://www.vyfi.io/) but now we moved to Cardano. Developers lerned plutus several months ago', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    161,  -- id
    (SELECT row_id FROM objective WHERE id=7 AND event=4), -- objective
    'Voluntary.Gold: a community network',  -- title
    'Worldwide cardano community lacks a decentralized platform for collaboration, leadership development, social interaction, & incentivisation.',  -- summary
    (SELECT category FROM objective WHERE id=7 AND event=4), -- category - VITSS Compat ONLY
    '92NXtk04YX4OSQC0IrzjLYoQhQ9FimUWvM8WFMglFEk=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBfxI', -- url
    '', -- files_url
    100, -- impact_score
    '{"solution": "Our federated network is being built for community members to self organize, train and support each other, and be rewarded for their service"}', -- extra
    'mani mejia', -- proposer name
    '', -- proposer contact
    'https://voluntary.gold', -- proposer URL
    'Our team of 3 has 31y in web technology; 56y in volunteer mgmt; 25y in entrepreneurship; 3y in crypto; a PhD in Org Leadership; plus you too', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    162,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Smart Contract Development Campaign',  -- title
    'User awareness, adoption, reduce barrier to entry. There are not as many people aware of smart contracts, their use and how to develop them.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'RVATjHDG0rzDvvSIlbfgo7zgFTg84L3ftAfGmhqMF9Q=', -- Public Payment Key
    '400000', -- funds
    'http://ideascale.com/t/UM5UZBfxK', -- url
    '', -- files_url
    120, -- impact_score
    '{"solution": "First, build a friendly, process for education of smart contracts, use-cases and their development. Second, develop SDKs and/or transpiler.."}', -- extra
    'Tom Reed', -- proposer name
    '', -- proposer contact
    'https://github.com/centrex', -- proposer URL
    'I have been a programmer and software engineer since 1995. I was first exposed to bitcoin and began mining it in 2010.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    163,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Petercoin',  -- title
    'Hey Peter! Cardano is missing an element of "fun" that would otherwise assist in introducing new users to the platform.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    '87MvSEXigzM4u1xvMaf13auzPdIIghwcJpxyVCet8IY=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBfxL', -- url
    '', -- files_url
    100, -- impact_score
    '{"solution": "PeterCoin would help to deliver this missing whimsicality, creating utilities such as PeterSwap and decentralized games to introduce users."}', -- extra
    'Petertoshi Nakamoto', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'We are a team of dedicated developers working to realize our vision of a decentralized future, and introducing new users is our top priority', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    164,  -- id
    (SELECT row_id FROM objective WHERE id=7 AND event=4), -- objective
    'Help University/hacker association',  -- title
    'An incentive to use Cardano has to be created for groups that are creating and experimenting with new technologies.',  -- summary
    (SELECT category FROM objective WHERE id=7 AND event=4), -- category - VITSS Compat ONLY
    '0fwwBhslvSJJ8Ko/UpPgL3/tmSASEOCOhwhj+moDJIs=', -- Public Payment Key
    '10000', -- funds
    'http://ideascale.com/t/UM5UZBfxP', -- url
    '', -- files_url
    233, -- impact_score
    '{"solution": "Provide useful real case examples and hints to use Cardano. Those hints can be used for hackathons and university groups/projects."}', -- extra
    'Async', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Bachelor of Science, applied Computer Science

2+ Years professional experience as a Software developer

5+ Years experience in SEO, marketi', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    165,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Mental Health Dapp on the go',  -- title
    'A mental health assistant app that stays with you on the go.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'Us+KYsBAGJNKVBLr9fwglPbFp3XTqMzX23vChuMTH7A=', -- Public Payment Key
    '25000', -- funds
    'http://ideascale.com/t/UM5UZBfxU', -- url
    '', -- files_url
    257, -- impact_score
    '{"solution": "Mental health is one of the most overlooked aspects of most of our lives that we forget to check up on ourselves, our mental well-being."}', -- extra
    'drkannobeck', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'UX/UI Designer', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    166,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Tokenized ADA for Yield Farming',  -- title
    'Huge amounts are locked up in DeFi yield farms on Eth and BSC. They don''t want to move to Cardano yet because of lesser yield opportunities.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    '6iNvuLvUqSwWuDFb5XcD0K7/5LiCAnbeLmGZwzlLJUQ=', -- Public Payment Key
    '80000', -- funds
    'http://ideascale.com/t/UM5UZBfxk', -- url
    '', -- files_url
    262, -- impact_score
    '{"solution": "A token with 1:1 convertibility with ADA, allowing holders to keep staking their underlying ADA while also yield farming using the tokens."}', -- extra
    'Nim', -- proposer name
    '', -- proposer contact
    'https://staking.rocks', -- proposer URL
    'I''m a web developer with 8+ years experience building apps in JS, Python and PHP. Currently also operating PHRCK stake pool in Philippines.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    167,  -- id
    (SELECT row_id FROM objective WHERE id=7 AND event=4), -- objective
    'University Cardano Research Program',  -- title
    'Creating new Cardano education and learning centres and labs is a big challenge which involves many aspects like location and equipments',  -- summary
    (SELECT category FROM objective WHERE id=7 AND event=4), -- category - VITSS Compat ONLY
    'OAonuYQpqO3k6vxq+Q0imdUa1aPV0FcVaBLBtOvWpTg=', -- Public Payment Key
    '15000', -- funds
    'http://ideascale.com/t/UM5UZBfxl', -- url
    '', -- files_url
    200, -- impact_score
    '{"solution": "We can speed up the learning and adoption of Cardano by using existing universities labs, equipments and networks."}', -- extra
    'Denison Luz', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Senior User Interface Developer with over 10 years of experience in web development and team management.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    168,  -- id
    (SELECT row_id FROM objective WHERE id=7 AND event=4), -- objective
    'Cardano Adoption in D.R. of Congo',  -- title
    'Bitcoin scams have led the government to take a negative stance on crypto currencies that result in bans and discourage blockchain adoption.',  -- summary
    (SELECT category FROM objective WHERE id=7 AND event=4), -- category - VITSS Compat ONLY
    '7zVxqWAr14Rk2QuySFhXmL+AanbG4FiTyh1jceeN9SA=', -- Public Payment Key
    '3000', -- funds
    'http://ideascale.com/t/UM5UZBfxw', -- url
    '', -- files_url
    438, -- impact_score
    '{"solution": "Create a local *Blockchain* *Hub for Solution Design*, for both developer and policy-maker onboarding."}', -- extra
    'fsamvura', -- proposer name
    '', -- proposer contact
    'https://play.google.com/store/apps/details?id=com.AgroApp.client.AgroApp', -- proposer URL
    ' Junior Haskell teacher

 Family relations with local lawmaker

 Access to WADA Network ressources

 Community Engagement & Outreach', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    169,  -- id
    (SELECT row_id FROM objective WHERE id=4 AND event=4), -- objective
    'Cardanotes',  -- title
    'Cardano needs a platform where great concepts meet great skills. The iPod required the vision of Steve Jobs and the engineers who built it.',  -- summary
    (SELECT category FROM objective WHERE id=4 AND event=4), -- category - VITSS Compat ONLY
    'Ne2GZuh55sRwbqi5dfstulXAkNO1eCfCz181MBJzqaA=', -- Public Payment Key
    '22000', -- funds
    'http://ideascale.com/t/UM5UZBfx9', -- url
    '', -- files_url
    156, -- impact_score
    '{"solution": "A website with breakout rooms where ideas for Dapps can be shared with programmers and programmers of various skills can tutor one another."}', -- extra
    'Chris Ossman', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'As an engineer for the past 20+ years and a developer for the past 2, I bring the experience of uniting a team for a common goal.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    170,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Music Database MIDI blockchain',  -- title
    'The way in wich we can relation with music actually is very limited.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'awIHp1oUBRGSMtILAOVPGCoBuUxPhqhLlrndKfKwPeg=', -- Public Payment Key
    '5000', -- funds
    'http://ideascale.com/t/UM5UZBfyL', -- url
    '', -- files_url
    100, -- impact_score
    '{"solution": "Create an application where we can share and search for music with freedom something like musescore.com + Synthesia but in blockchain"}', -- extra
    'plcarmona', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I start play piano like 4 years ago totally by selflearning, and i allways search for a simple way to search and make music.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    171,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Stake Pool Operations Dashboard',  -- title
    '24/7 uptime requires alerting and monitoring to maintain. How can we give new operators awareness of top health metrics for their pool.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'wqI+DPQTsDqG4xU0iWuVhEqk/ySthXJv4tFQIO/SgpA=', -- Public Payment Key
    '8000', -- funds
    'http://ideascale.com/t/UM5UZBfyV', -- url
    '', -- files_url
    125, -- impact_score
    '{"solution": "We can provide operators a data exporter to be consumed by their dashboarding tool of choice."}', -- extra
    'abhiaiyer91', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I have been engineering in the cloud systems space for multiple years. We use alerting and metrics to keep SaaS products up 99% of the time.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    172,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Voluntary.Gold: a community network',  -- title
    'Worldwide cardano community lacks a decentralized platform for collaboration, leadership development, social interaction, & Incentivisation.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'dGo0g8naCaT838Gsl37bjQDJZ+FUar8bE8qXP8bDcIM=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBfyc', -- url
    '', -- files_url
    217, -- impact_score
    '{"solution": "Our federated network is being built for community members to self organize, train and support each other, and be rewarded for their service"}', -- extra
    'mani mejia', -- proposer name
    '', -- proposer contact
    'https://voluntary.gold', -- proposer URL
    'Our team of 3 has 31y in web technology; 56y in volunteer mgmt; 25y in entrepreneurship; 3y in crypto; a PhD in Org Leadership; plus you too', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    173,  -- id
    (SELECT row_id FROM objective WHERE id=4 AND event=4), -- objective
    'Voluntary.Gold: a community network',  -- title
    'Worldwide cardano community lacks a decentralized platform for collaboration, leadership development, social interaction, & incentivisation.',  -- summary
    (SELECT category FROM objective WHERE id=4 AND event=4), -- category - VITSS Compat ONLY
    'v7T3BL1oGWlDpeAuKFRuMX7BzJGCHDHqHVshQi1LCSM=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBfyo', -- url
    '', -- files_url
    223, -- impact_score
    '{"solution": "Our federated network is being built for community members to self organize, train and support each other, and be rewarded for their service"}', -- extra
    'mani mejia', -- proposer name
    '', -- proposer contact
    'https://voluntary.gold', -- proposer URL
    'Our team of 3 has 31y in web technology; 56y in volunteer mgmt; 25y in entrepreneurship; 3y in crypto; a PhD in Org Leadership; plus you too', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    174,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Distributed financial support',  -- title
    'Small communities in emerging economies do not have adequate access to financial services.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'iYrdmj0wSQCoJPQA3FH3ZbzPL35AbSC5+YRvPvD4zvQ=', -- Public Payment Key
    '40000', -- funds
    'http://ideascale.com/t/UM5UZBfyp', -- url
    '', -- files_url
    167, -- impact_score
    '{"solution": "We plan to facilitate decentralised access to credit for individuals who otherwise don''t have access to traditional credit sources."}', -- extra
    'andy.bowness', -- proposer name
    '', -- proposer contact
    'https://github.com/kryptt', -- proposer URL
    'Applicant 1: Product Owner in Financial Services https://bit.ly/3bUMF75

Applicant 2: Lead developer https://bit.ly/3bROTUV', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    175,  -- id
    (SELECT row_id FROM objective WHERE id=6 AND event=4), -- objective
    'Voluntary.Gold: a community network',  -- title
    'Worldwide cardano community lacks a decentralized platform for collaboration, leadership development, social interaction, & incentivisation.',  -- summary
    (SELECT category FROM objective WHERE id=6 AND event=4), -- category - VITSS Compat ONLY
    'UHfad8PvUexx0a2IrGhCqMLe8QGHNXdF9xJ6WrxlueI=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBfyz', -- url
    '', -- files_url
    275, -- impact_score
    '{"solution": "Our federated network is being built for community members to self organize, train and support each other, and be rewarded for their service"}', -- extra
    'mani mejia', -- proposer name
    '', -- proposer contact
    'https://voluntary.gold', -- proposer URL
    'Our team of 3 has 31y in web technology; 56y in volunteer mgmt; 25y in entrepreneurship; 3y in crypto; a PhD in Org Leadership; plus you too', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    176,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'PoolTool-Testnet Support',  -- title
    'Samuel at IOHK has requested that we have a version of PoolTool for the Testnet as well as mainnet.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'gg0tiN2NuEL07Or47RRIPESsu9HV3mkyH3RsYav5ZCM=', -- Public Payment Key
    '1500', -- funds
    'http://ideascale.com/t/UM5UZBfy6', -- url
    '', -- files_url
    453, -- impact_score
    '{"solution": "We have spun up the testnet on the main pooltool platform with limited features and would like to have the fund cover our base expenses."}', -- extra
    'Umed--[SKY] SkyLight Pool', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'We are the team behind pooltool.io', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    177,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Delegation Tracker',  -- title
    'Onboarding barrier for ADA delegators is large, existing solutions cause centralization of stake.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'X5RXN5Z4yclucohxLnkan2rXaihVKnOe9Pr9oSdgIQ8=', -- Public Payment Key
    '18550', -- funds
    'http://ideascale.com/t/UM5UZBfzB', -- url
    '', -- files_url
    183, -- impact_score
    '{"solution": "Improve the customer experience of onboarding and staking and provide a solution that will allow for better distribution of stake."}', -- extra
    'Ron ADA4Profit', -- proposer name
    '', -- proposer contact
    'https://delegationtracker.com', -- proposer URL
    'We are a software company. Our dev team has many years experience in building complex scalable cloud native solutions.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    178,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Cardano with Ñ / Cardano in Spanish',  -- title
    'The lack of knowledge about Cardano is common at non-native speaking countries where fewer resources are available in their own language.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'I3gfxRCUBU+2AEHCGPOMQgI83cgzOdwFle94rSyQB7E=', -- Public Payment Key
    '3000', -- funds
    'http://ideascale.com/t/UM5UZBfzL', -- url
    '', -- files_url
    256, -- impact_score
    '{"solution": "We plan to produce a series of short videos about Cardano in Spanish. The target of these videos are people who don''t know Cardano."}', -- extra
    'Javier Urrestarazu', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'We are learning how to build and maintain our own stake pool and we would like to go for more technical projects when we are ready for it.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    179,  -- id
    (SELECT row_id FROM objective WHERE id=4 AND event=4), -- objective
    '+WADA Outreach=Smart Catalyst Users',  -- title
    'WADA''s outreach is currently limited by their web app capacity. They need their own solid integrated management system to optimize outreach.',  -- summary
    (SELECT category FROM objective WHERE id=4 AND event=4), -- category - VITSS Compat ONLY
    'YhA2zxjci6mFVBjIIOhEWyGJewfQ3Tikhk9Wftc2mGw=', -- Public Payment Key
    '9000', -- funds
    'http://ideascale.com/t/UM5UZBfzO', -- url
    '', -- files_url
    333, -- impact_score
    '{"solution": "Build WADA a solid, scalable, distributed, proof of concept system/ Dapp from the ground up using Servant Server, Haskell, and Elm."}', -- extra
    'Megan Hess', -- proposer name
    '', -- proposer contact
    'https://github.com/diaspogift/lost-and-found-inventory', -- proposer URL
    'Disruptive IT Cameroon+ WADA: Team of passionate software architects/developers with strong foundations in functional programming paradigm', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    180,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Vy.Finance DeFi Protocol',  -- title
    'DeFi has a few issues right now, one of which being gas fees, another being a lack of diversity into things besides just currencies.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    '/MRu2xJuAxZp2UMwzHV4pMLX+ymjM8jhV54VVt+Z2uY=', -- Public Payment Key
    '25000', -- funds
    'http://ideascale.com/t/UM5UZBfzQ', -- url
    '', -- files_url
    124, -- impact_score
    '{"solution": "Cardano, with bridging and wrapping to as many other chains as possible. The second issue is solved by allowing users to stake into our PTF."}', -- extra
    'Jack Kochen', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Worked with Aurox Trading Terminal which has now turned DeFi. Also DBET, VET/VEN, MDX, and many more companies. Vy.Fi is entirely in-house.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    181,  -- id
    (SELECT row_id FROM objective WHERE id=6 AND event=4), -- objective
    'Cardano Mobile App',  -- title
    'Members need a structured format to easily on board to the Project Catalyst project in a format they are familar with.',  -- summary
    (SELECT category FROM objective WHERE id=6 AND event=4), -- category - VITSS Compat ONLY
    'fg+PowJXR+XbFmGooB4B/T8HugpielkND5GYJxYzaiY=', -- Public Payment Key
    '5000', -- funds
    'http://ideascale.com/t/UM5UZBfzS', -- url
    '', -- files_url
    318, -- impact_score
    '{"solution": "A Mobile App to educate on Project Catalyst, manage tasks for roles being worked on. Members will be rewarded with Catalyst tokens."}', -- extra
    'Chris Patten-Walker', -- proposer name
    '', -- proposer contact
    'https://cardanomobile.com', -- proposer URL
    '20+ years in the Financial Industry running teams and developing applications for Top Tier Banks. 5 years building blockchain apps and APIs.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    182,  -- id
    (SELECT row_id FROM objective WHERE id=6 AND event=4), -- objective
    'Project Catalyst Landing Page',  -- title
    'There is no central place for new and existing community members of Project Catalyst to find the info and links they need.',  -- summary
    (SELECT category FROM objective WHERE id=6 AND event=4), -- category - VITSS Compat ONLY
    'Oi8dBUYu8M3tmtn43/W6SuH69h1a0fusrezGgqHVNjk=', -- Public Payment Key
    '10000', -- funds
    'http://ideascale.com/t/UM5UZBfz6', -- url
    '', -- files_url
    476, -- impact_score
    '{"solution": "Build a community driven Community Landing Page that links to the various material relating to Project Catalyst."}', -- extra
    'Philip Khoo', -- proposer name
    '', -- proposer contact
    'https://github.com/Project-Catalyst/catalyst', -- proposer URL
    'A team of community members. Phil Khoo-project lead / Project Catalyst Community Advisor, Jacob Abel lead developer, mwojtera dev ops & Tevo', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    183,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Fair Food Ordering & Delivery',  -- title
    'delivery workers get 100% of gratuity leaving restaurant workers out and fuel/time efficiency is ignored. unwanted fees to merchants/users.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'yaxcBMh6AzbWUYkzz5j+RH48l5h4OuVclTgrGKGF/8k=', -- Public Payment Key
    '108', -- funds
    'http://ideascale.com/t/UM5UZBfz7', -- url
    '', -- files_url
    171, -- impact_score
    '{"solution": "eco-friendly hive food delivery dapp with multi-tip splits. help mass adoption - millions of new wallets(workers+users) / $300B transaction"}', -- extra
    'prasanna malla', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'developer+Postmates driver+Cardano believer. looking for collaborators to build a fair product with huge everyday use case.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    184,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=4), -- objective
    'Multilingual resources',  -- title
    'How can we motivate non english speakers to learn and grow Cardano ecosystem in the next 3-6 months?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=4), -- category - VITSS Compat ONLY
    'pYkebHCGhrLza2rwn6xdV6BmI4cv9y8CAGkeYjV5mRc=', -- Public Payment Key
    '75000', -- funds
    'http://ideascale.com/t/UM5UZBf0D', -- url
    '', -- files_url
    400, -- impact_score
    '{"brief": "Problem:\r\n\r\nThe english language could be a barrier for a lot of people around the world when they try to learn about Cardano and blockchain technology.\r\n\r\nAt least in my native language (spanish) there are no resources for people who might want to learn how to code smart contracts with Plutus or Marlowe for example.\r\n\r\nSolution:\r\n\r\nWe can set a challenge with a modest budget ($50.000-100.000 USD) for people to propose solutions on how to increase adoption among non native english speakers.\r\n\r\nExamples:\r\n\r\n Create (or translate) courses in non english languages, to increase the amount of resources in a variety of topics for people who don''t speak english.\r\n\r\n Fund an educational organization focused on bringing free resources about blockchain technology and Cardano solutions in their local community.\r\n\r\n Any other idea people might have to increase adoption of non english speakers.", "importance": "Multilingual platforms will allow non english speakers to learn about Cardano and increase its adoption worldwide.", "goal": "Increased adoption and community engagement of non english speakers.\r\n\r\nIncreased amount of resources in a variety of languages.", "metrics": "At the end of this challenge, we will be asking ourselves: Did we manage to make it easier for non english speakers to build and grow the Cardano community in their local areas?\r\n\r\n Number of non english speakers joining Cardano community\r\n\r\n Number of courses translated to different languages, from programming to stakepool operation and so on.\r\n\r\n A sense of community and conversations. Having people around to talk with."}', -- extra
    'Tomas Garro', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    185,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'tokenize agricultural products',  -- title
    'Colombian farmers earn very little. people die of hunger in Colombia. investors are currently unable to trade agricultural digital assets',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'hoO42NCd7s0GnjlEm8u6Q5mzMoN+eziidijBimUC+7c=', -- Public Payment Key
    '100000', -- funds
    'http://ideascale.com/t/UM5UZBf0F', -- url
    '', -- files_url
    308, -- impact_score
    '{"solution": "Farmers receive token rewards for their work. people will have food. enable investors and individuals to trade agricultural digital assets"}', -- extra
    'jhonatan smith gutierrez torres', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'The idea is proposed by my family: we are 3 brothers and our mother: we are Lawyer, engineer and programmer, NFT artist and Cardano traders.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    186,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=4), -- objective
    'Functional Paradigm Onboarding ',  -- title
    'How to demonstrate Functional Programing Capabilities within your own existing management system structure?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=4), -- category - VITSS Compat ONLY
    '/H7Ea+xa2wNjIdBsxeooQguPbBQqRGuF/HD899AXDcI=', -- Public Payment Key
    '21000', -- funds
    'http://ideascale.com/t/UM5UZBf0I', -- url
    '', -- files_url
    293, -- impact_score
    '{"brief": "Creating a thoughtful system requires, patience, thoughtful people and efficient tools. Cardano itself is built in this way, and therefore our community should also reflect that thoughtful and diligent approach in our own enterprises.\r\n\r\nHaskell, as a programing language uses Lamda Calculus underneath, which makes it easier for developer to reason about their code.\r\n\r\nWhen it compiles, it works!", "importance": "We get super excited about ADA, but we forget how it was architected and built. Haskell is the real unsung hero behind Cardano", "goal": "Army of functional programmers (trained throughout app development process)  \r\n\r\nFully functional Web app built using Servant, Haskell, and Elm.", "metrics": " git hub repository commits\r\n\r\n shared scrum / Kanban board\r\n\r\n fully functional web app\r\n\r\n trained developers"}', -- extra
    'Megan Hess', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    187,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'NFT-DAO EZ-on',  -- title
    'Writing safe DApps today is too difficult and developers lack tools to easily build composable components.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    '2XXBwVNOT8cKxJJP+0DpLEzmRt5E9xz4yU/BimdGeX8=', -- Public Payment Key
    '63842', -- funds
    'http://ideascale.com/t/UM5UZBf00', -- url
    '', -- files_url
    208, -- impact_score
    '{"solution": "Partnering with MuKn.io/Glow to make building infrastructure fast, easy to write and audit to onboard developers new to the space."}', -- extra
    'rich', -- proposer name
    '', -- proposer contact
    'https://NFT-DAO.org', -- proposer URL
    'MuKn.io/Glow brings skills of cybersecurity, cryptography, distributed systems, systems programming, economic modeling & mechanism design.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    188,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Experimental Educational Videos',  -- title
    'Distributed ledger technology and the perceived benefits are not really understood by the general public, developers or entrepreneurs',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'T9tACBra0zi37TMAuzhISFfqC5CEHjjW6DPlCKXEYTI=', -- Public Payment Key
    '6000', -- funds
    'http://ideascale.com/t/UM5UZBf01', -- url
    '', -- files_url
    225, -- impact_score
    '{"solution": "Generate videos and other learning materials through direct mentorship with curious individuals"}', -- extra
    'timothy eichler', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    ' Self taught programmer

 4 years Agile software development

 Pair programming

 Mentoring developers w/o prior work experience', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    189,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Native Asset and Metadata app',  -- title
    'With the release of Mary HFC, native assets are now available. A tool is needed to assist asset creators define token details and metadata',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'B7fQpTfi/UvEGpsXNS2vu97P/9L/pRLTbEhemyepJ2E=', -- Public Payment Key
    '20000', -- funds
    'http://ideascale.com/t/UM5UZBf1F', -- url
    '', -- files_url
    133, -- impact_score
    '{"solution": "A website and dApp to provide step by step walk through process of defining a native token / asset, token policy and asset metadata"}', -- extra
    'Joseph P', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Mobile application developer, enterprise systems design, experience with process workflow and technical documentation', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    190,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'ADA Subscription Payments',  -- title
    'Staking ADA should not be the only way to earn ADA.  
Content creators are charged even 30% in fees by Google, Microsoft, Apple & Sony.\[1\]',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'M+8XbX36yuT1QQaoTxDyRlx8ug2kbtGwN/aL1d2CL+E=', -- Public Payment Key
    '45000', -- funds
    'http://ideascale.com/t/UM5UZBf1S', -- url
    '', -- files_url
    217, -- impact_score
    '{"solution": "Increase content creators'' monetization options: subscription payments in ADA.\r\n\r\nAccess content for short period of time or selected articles"}', -- extra
    'Greg', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Software Engineer with over 10 years of commercial experience based in London. Recently finished fintech course at Oxford University.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    191,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Tokenized Property Investment DApp',  -- title
    '$200 Trillion in the real estate market is currently inaccessible to many millions of willing investors, we can solve that with blockchain',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'HkeYt/iC683DSdjpz+nvIYY4FTkEqWMJCAGFI4du5Uk=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBf1T', -- url
    '', -- files_url
    175, -- impact_score
    '{"solution": "We are proposing a tokenized network built on the Cardano blockchain, where anyone can invest in the property at any level, worldwide."}', -- extra
    'Henry', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'We are a team of:

Architect

Innovator

Property Expert

Sustainable and Environmental Engineer

We have the experience and know-how', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    192,  -- id
    (SELECT row_id FROM objective WHERE id=4 AND event=4), -- objective
    'Calling All Deadheads!',  -- title
    'Catalyst sits at the forefront of innovation in distributed decision-making. We don''t have any historical models to guide us. Or do we?',  -- summary
    (SELECT category FROM objective WHERE id=4 AND event=4), -- category - VITSS Compat ONLY
    'jOWoXN6c71BJlKYKHM4Afw4g5hMKUXW1ceMSm/NxSZo=', -- Public Payment Key
    '5000', -- funds
    'http://ideascale.com/t/UM5UZBf1X', -- url
    '', -- files_url
    300, -- impact_score
    '{"solution": "Pre-Internet, the Grateful Dead created and maintained a distributed and decentralized, yet deeply committed community. What can we learn?"}', -- extra
    'Michael McNulty', -- proposer name
    '', -- proposer contact
    'https://www.youtube.com/watch?v=KKkr4w22uPI', -- proposer URL
    'I''m just a dude that really likes the Grateful Dead.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    193,  -- id
    (SELECT row_id FROM objective WHERE id=6 AND event=4), -- objective
    'Teaming to integrate multiple ideas',  -- title
    'Individuals only give one perspective on ideas for the future of Cardano and may lose interest overtime.',  -- summary
    (SELECT category FROM objective WHERE id=6 AND event=4), -- category - VITSS Compat ONLY
    'e6Srx08LFb6vMfx8FZoksaBS/fKdMnxhk90h6zX7tJo=', -- Public Payment Key
    '1000', -- funds
    'http://ideascale.com/t/UM5UZBf1Y', -- url
    '', -- files_url
    122, -- impact_score
    '{"solution": "By allowing teams, the community will be more inclined to contribute their idea and become an active voice on initiatives concerning Cardano"}', -- extra
    'Karam H', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'XPRIZE is similar where teams are able to work together to create something innovative, where people have roles according to their strengths', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    194,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=4), -- objective
    'DLT Entrepreneurship Toolbox',  -- title
    'How can early stage entrepreneurs develop their skills to execute & communicate great ideas so voters/stakeholders value the project',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=4), -- category - VITSS Compat ONLY
    'HbcZFmDt/8892/TwpnBo5QavLxCOCuOpXbaOjxzo7AQ=', -- Public Payment Key
    '150000', -- funds
    'http://ideascale.com/t/UM5UZBf1b', -- url
    '', -- files_url
    417, -- impact_score
    '{"brief": "Whatever your background, confidently communicating all the important elements of an early stage venture can be overwhelming for inexperienced entrepreneurs.\r\n\r\nMore should be done to help an increasingly diverse group of Project Catalyst entrepreneurs, at different stages of self-development, plus their teams and stakeholders to collaborate and communicate effectively.\r\n\r\nThis is important to Project Catalyst as, over time, the strongest ideas will be those that are the most coherently communicated and which ultimately resonate clearly with voters - particularly Expert evaluators!\r\n\r\nThis challenge seeks to help inexperienced entrepreneurs to learn and apply vital entrepreneurship skills so they can collaborate with teams and stakeholders on meaningful Catalyst projects. This also helps ensure sustainable impact and a positive return on investment and intention is made.\r\n\r\nBusiness accelerators provide entrepreneurs with: \"a wide range of opportunities for innovation in their industry and an alternative to traditional entrepreneurial education\" (Miller & Bound, 2011).\r\n\r\nHowever this is still an education which too many entrepreneurs miss out on! Meaning many talented engineers and ambitious entrepreneurs alike often struggle to get the resources required just to get started. And too many projects die before they are born!\r\n\r\nThe focus of this challenge should be on developing entrepreneurship skills in:\r\n\r\n\\* entrepreneurial communications  \r\n value proposition design and testing  \r\n\\* financial literacy for business ideation  \r\n measuring sustainable development impacts and growth\r\n\r\nThis could be for developing their technology projects and proposals for (or during) Project Catalyst, the most disruptive business accelerator on the planet today.\r\n\r\nThis challenge seeks novel solutions that equips users with a simple to grasp digital entrepreneurship toolset and knowledge base to apply to their own venture plans. Digital learning, collaboration, and guiding entrepreneurs to self-determine goals and demonstrate learnt skills are all core to this challenge.\r\n\r\nInteraction with Cardano''s Treasury system and/or Native Asset Tokens could also play a significant role in basic prototyping or for entrepreneurship education resources to be created and made relevant to decentralised use cases. Though challenge submissions would also accept non-interactive design ideas.\r\n\r\nHowever proposals should ultimately help startups take all the necessary steps to refine perhaps many ideas along the process so they become confident presenters and evaluators of their business'' principles, uniqueness, business model, and project objectives. Plus enable valid ways to demonstrate sustainable progress during the lifetime of the Catalyst funding and beyond.", "importance": "Entrepreneurs strengths are ambition and vision but often lack structure to communicate ideas clearly enough to win voter hearts & minds", "goal": "Higher quality of proposal submitted to Catalyst so entrepreneurs become confident presenting project principles, USP, strategy, objectives", "metrics": "Number of users, using the tools\r\n\r\nDifference in score of applicants using the toolbox and those not using the tools.\r\n\r\nGaps that are filled in the entrepreneurial skills base of project teams from the start of ideation through project delivery.\r\n\r\nSustainability of venture plans *beyond* the scope of Catalyst development project."}', -- extra
    'Kriss', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    195,  -- id
    (SELECT row_id FROM objective WHERE id=5 AND event=4), -- objective
    'WorldWide ADA/local fiat possible',  -- title
    'Cardano should be worthy of baking the unbank but people in several countries of the world CAN''T use their own fiat to buy ADA yet',  -- summary
    (SELECT category FROM objective WHERE id=5 AND event=4), -- category - VITSS Compat ONLY
    'ZTqyXFMjoRCyWtd3xuALN+gDJAKcW8TtOW9pygmdPbs=', -- Public Payment Key
    '3000', -- funds
    'http://ideascale.com/t/UM5UZBf15', -- url
    '', -- files_url
    175, -- impact_score
    '{"solution": "Defining, in coordination with Cardano, a decentralized platform (not only tech) that helps entrepreneurs to create ADA/local fiat exchanges"}', -- extra
    'Miguel Barrera', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I''m a CS Engineer and also I have a diploma in Biz Management. 20+ yrs of experience as a PM, Biz Analyst, Consultant in many Multinationals', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    196,  -- id
    (SELECT row_id FROM objective WHERE id=7 AND event=4), -- objective
    'Al-Baari. The Creator. ',  -- title
    'Blockchain and multiple applications of Cardano, and this new era of technology, are foreign for the majority of the globe.',  -- summary
    (SELECT category FROM objective WHERE id=7 AND event=4), -- category - VITSS Compat ONLY
    '994DYDWo1LCfHKdGmLcWBxtOU64k+Cbfp9CDRBJ75IU=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBf18', -- url
    '', -- files_url
    233, -- impact_score
    '{"solution": "Creating a model, in which Cardano funds R+D cells, by \"renting out\" multidisciplinary classes in Universities. Using a especificmethodology"}', -- extra
    'jorgeae17', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I''m an anthropologist, who has been working on Pedagogy, Innovation and Marketing my whole life. Creating culture through education.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    197,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'ADA-backed algorithmic stablecoin',  -- title
    'Upcoming defi projects for cardano will require one or more stablecoins for traders to prevent capital loss due to price volatility.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'FkulNk1N01WXWRazWNUN7yhSxkH3mVYN4tS60P2XSJE=', -- Public Payment Key
    '5001', -- funds
    'http://ideascale.com/t/UM5UZBf2I', -- url
    '', -- files_url
    125, -- impact_score
    '{"solution": "One solution to the problem is to implement a ADA-backed algorithmic stable coin by forking and deploying an existing project."}', -- extra
    'carlosbf94', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Computer Science degree.

Experience working in embedded software development using javascript,and c++.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    198,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Indigenous Art Authenticity',  -- title
    'COVID pandemic pushed indigenous artists to sell online. Ensuring authenticity of indigenous goods sold online is crucial to their survival.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'yLYfHpyQ8oXdJZPtHERPKJ4UoV3uQ9GoPpnwQe3xBTw=', -- Public Payment Key
    '35000', -- funds
    'http://ideascale.com/t/UM5UZBf2M', -- url
    '', -- files_url
    386, -- impact_score
    '{"solution": "Convert IndigeneArts.com to be a niche NFT marketplace featuring indigenous art, certified by digital certificates of authenticity."}', -- extra
    'Dmitri Safine [DDAY]', -- proposer name
    '', -- proposer contact
    'https://IndigeneArts.com', -- proposer URL
    'unCommonThread.biz team has built a successful IndigeneArts.com marketplace featured in Forbes.

4 people in Canada

6 people in Ukraine', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    199,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Live Coding Mentor Marketplace',  -- title
    'Coding roadblocks inhibit or slow down blockchain project completion & learning. Docs can be daunting & unhelpful compared to live help.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'DLJKlVnKHWcKo+yy/GoynIvLykwvdR0PY/nymVbcy74=', -- Public Payment Key
    '13000', -- funds
    'http://ideascale.com/t/UM5UZBf2R', -- url
    '', -- files_url
    411, -- impact_score
    '{"solution": "A marketplace for students & founders to find mentors and/or pro coders to help them through issues, tasks, & train in live coding sessions"}', -- extra
    'Travis', -- proposer name
    '', -- proposer contact
    'https://github.com/tmdcpro/Prorata', -- proposer URL
    'Travis Cook: UI/UX Designer, Front-end Dev, & Product Owner

Erfun Tavakoli: Fullstack Developer, React, Typescript, Functional Programming', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    200,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'CRITQ~Art Criticism NFT Marketplace',  -- title
    'Artists and buyers have difficulty finding the proper valuation of art and NFTs. Marketplaces need to integrate the process of *art* *criticism*',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    '7/34Wn4Um7cQajOseCSYHklKeZ8X/Mhq8tmeJnjdoiU=', -- Public Payment Key
    '20000', -- funds
    'http://ideascale.com/t/UM5UZBf2g', -- url
    '', -- files_url
    420, -- impact_score
    '{"solution": "An NFT Art marketplace centred around *crowdsourced* *art criticism* rather than exclusive juried submissions or nonsensical bidding ''wars''"}', -- extra
    'Travis', -- proposer name
    '', -- proposer contact
    'https://github.com/tmdcpro/CRITIQ', -- proposer URL
    'Travis Cook: Front-end dev, Art Gallery co-founder  

Edno Mesquita: Fullstack, Blockchain Dev  

David Fatimehin: Business, Gallery co-founder', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    201,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'dRelic',  -- title
    'Traditional bidding methods are uninteresting and do not take advantage of the power the blockchain offers.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    '3tL7fN2ls1UtkTbKTDhHYQByyf+ZKcmOSaz1ZG9qWR4=', -- Public Payment Key
    '5000', -- funds
    'http://ideascale.com/t/UM5UZBf2k', -- url
    '', -- files_url
    292, -- impact_score
    '{"solution": "Through gamifying a curated drop of selected artists, utilizing pre-determined rules that are executed automatically via smart contracts."}', -- extra
    'drelicnft', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Stake Pool Operator (CHEST)

20 years of collective coding experience', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    202,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    '"ADA" Visual Scripting VSCode Ext.',  -- title
    'The complexity of blockchain tech can make it difficult to predict & debug dapp code. The process could be simplified by visual abstraction.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'DiGeeJ75u/1nEyMfOsAIeD2jqv4CWtZmkGaJ3xkbu+U=', -- Public Payment Key
    '6000', -- funds
    'http://ideascale.com/t/UM5UZBf2u', -- url
    '', -- files_url
    387, -- impact_score
    '{"solution": "A VSCode plugin for Plutus/Haskell to visualize on-chain transactions, smart contracts, state machine, etc. using nodegraph ''flow'' diagrams."}', -- extra
    'Travis', -- proposer name
    '', -- proposer contact
    'https://github.com/tmdcpro/ADA-vscode-visual-scripting-interface', -- proposer URL
    'I have experience in UI/UX & graphic design, prototyping, animation/film/video, front-end dev, functional programming & design patterns.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    203,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Plutus Certification Program',  -- title
    'Companies looking for developers to help with building smart contacts won''t know if the developer know what they are doing.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'KUTToVy6xUQ2zlhSrZR0zyDhUmv2sZZTZYjYzEryVTM=', -- Public Payment Key
    '400000', -- funds
    'http://ideascale.com/t/UM5UZBf24', -- url
    '', -- files_url
    122, -- impact_score
    '{"solution": "Create a certification program around the entire smart contact / Plutus system."}', -- extra
    'Tom Reed', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I''ve been a software developer for over 25 years.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    204,  -- id
    (SELECT row_id FROM objective WHERE id=5 AND event=4), -- objective
    'Crypto Blog & Cardano Podcast',  -- title
    'Many people still don''t understand blockchain and are afraid of it the big bad Cryptos. Cardano may as well be a type of cheese to them.',  -- summary
    (SELECT category FROM objective WHERE id=5 AND event=4), -- category - VITSS Compat ONLY
    'aXltsls21IJ0f9qCpkl2J+RIZjqzLitbT1dZOM4EfL0=', -- Public Payment Key
    '4000', -- funds
    'http://ideascale.com/t/UM5UZBf3J', -- url
    '', -- files_url
    206, -- impact_score
    '{"solution": "I have recently published a blog and one of its focuses will be introducing blockchain & cryptos to the masses... and Cardano evangelism."}', -- extra
    'Travis', -- proposer name
    '', -- proposer contact
    'http://static-press.com', -- proposer URL
    'I am a writer.

https://www.linkedin.com/in/travis-cook-3a980a7/', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    205,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Bridging Cardano to Onomy & Beyond',  -- title
    'Value must easily flow in & out of the ADA economy. The cost of no bridges is the inability for ADA to access established DeFi ecosystems.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    '0EpgyTE/2W1lgMlNxhEBKaUO7KXEOk2Mye3qbswElFM=', -- Public Payment Key
    '25000', -- funds
    'http://ideascale.com/t/UM5UZBf3V', -- url
    '', -- files_url
    360, -- impact_score
    '{"solution": "Create a bridge between Cardano and Onomy (Cosmos IBC Framework Integrated) to open value transfer to and from many blockchain economies"}', -- extra
    'Lalo', -- proposer name
    '', -- proposer contact
    'https://docs.onomy.io', -- proposer URL
    ' Gravity Bridge Implementation (Cosmos to ETH)

 Partnered with RINA (Catalyst Funded) & Cardano SPOs to bootstrap bridge.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    206,  -- id
    (SELECT row_id FROM objective WHERE id=5 AND event=4), -- objective
    'A Cardano editorial news website',  -- title
    'We need to have greater news coverage across the internet related to the ongoing problems that Cardano infrastructure is trying to solve',  -- summary
    (SELECT category FROM objective WHERE id=5 AND event=4), -- category - VITSS Compat ONLY
    'AJRMGPPnMF1kmbv16Pd+kC5yJGJv/ZhPRlsELEpn+2s=', -- Public Payment Key
    '6000', -- funds
    'http://ideascale.com/t/UM5UZBf3b', -- url
    '', -- files_url
    211, -- impact_score
    '{"solution": "A permanent news site that gives coverage to what is happening on catalyst including Cardano and current project updates to attract interest"}', -- extra
    'Inthedow3', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Have experience writing editorial newspaper articles and have a deep interest in Cardano as well as website development experience', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    207,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Decentralization Aids: Geo & Infra',  -- title
    'Cardano has an elegant design behind it, but delegators and stake pool operators need access to information to capture the potential of it.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'HHbSu4L7P2mPUPz4eq88piiL9jaN7obxVLhnrMJOcus=', -- Public Payment Key
    '25000', -- funds
    'http://ideascale.com/t/UM5UZBf3y', -- url
    '', -- files_url
    413, -- impact_score
    '{"solution": "Web app which displays a pools relay location, stake and infrastructure provider, with the intent of spreading Cardano far and wide."}', -- extra
    'Alexander Watanabe', -- proposer name
    '', -- proposer contact
    'https://www.monadpool.com/cardano.html', -- proposer URL
    'Worked in R&D for past 8 years, 3 years ago I started training and deploying AI models.  
https://www.linkedin.com/in/alexander-watanabe/', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    208,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Cardano On-Chain Voting',  -- title
    'People desire a secure, reliable, and immutable source of truth when dealing with votes, polls, and elections.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'qeVDtHtE4xYilYCwjejzkCOwTDZ4HRkv7wnB3oMeD4s=', -- Public Payment Key
    '15000', -- funds
    'http://ideascale.com/t/UM5UZBf37', -- url
    '', -- files_url
    433, -- impact_score
    '{"solution": "Expand and grow our existing solutions to vote on and via the blockchain to provide secure, immutable information and results to votes."}', -- extra
    'Adam Dean', -- proposer name
    '', -- proposer contact
    'https://github.com/crypto2099', -- proposer URL
    'Developed, deployed, and successfully executed the first Cardano on-chain vote.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    209,  -- id
    (SELECT row_id FROM objective WHERE id=7 AND event=4), -- objective
    'Save Farmers in India Initiative!',  -- title
    'Farm Laws passed increase privatization and middlemen profits by curbing Farmer to consumer transactions. Farmers livelihood compromised.',  -- summary
    (SELECT category FROM objective WHERE id=7 AND event=4), -- category - VITSS Compat ONLY
    'uZL2qFi2q2tfXsAxv6Qz8xf3+RNvhb0hnHWB1e+fiFQ=', -- Public Payment Key
    '15000', -- funds
    'http://ideascale.com/t/UM5UZBf4U', -- url
    '', -- files_url
    232, -- impact_score
    '{"solution": "Spread awareness on Cardano and it''s functionality to solve Farmer funding problems with marked tokens and hence eliminate corrupt middlemen"}', -- extra
    'akashthyagaraja', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Team comprising of Software Engineers, MBA graduates and NGO volunteers with an agricultural lineage striving to accomplish a common goal.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    210,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'M-Pesa payment rails integration',  -- title
    'M-Pesa is the most successful mobile-phone financial service in the developing world. Integrating with them will make Ada more accessible.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'AMTQL+VmoJ+PAcuMcXsn80r0gOx10D72PKo2aNkFSwg=', -- Public Payment Key
    '20000', -- funds
    'http://ideascale.com/t/UM5UZBf4b', -- url
    '', -- files_url
    179, -- impact_score
    '{"solution": "We plan to build an integration with the API that M-Pesa provides to make it easier to transact using Ada."}', -- extra
    'Randy Chung', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Our team consists of professionals in software development and business development, with existing ties to M-Pesa''s use in Kenya', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    211,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'DARP: Cardano Address Name Service',  -- title
    'Developers need ways to manage addresses in their dApps, for user experience and backend integrations (e.g. smart contracts, NFT data)',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'DVeUYf0D+6/NwbYiOeR0/NLBFg3hJ0ukR5yl9ZVJ7IE=', -- Public Payment Key
    '57500', -- funds
    'http://ideascale.com/t/UM5UZBf4d', -- url
    '', -- files_url
    283, -- impact_score
    '{"solution": "The Decentralised Address Resolution Protocol (DARP) builds on Atala PRISM''s foundations to allow easy to remember address names to be used."}', -- extra
    'Phil Lewis', -- proposer name
    '', -- proposer contact
    'https://docs.darp.tech/', -- proposer URL
    'Please review my experience on LinkedIn at https://www.linkedin.com/in/phillewisit/', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    212,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'OctoWars - a game platform',  -- title
    'Cardano doesn''t provide a simple and elegant way to build trading cards and turn based strategies.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'YFi3r5raWzdAFjm5thIslxn84oVweM+C1ma8MecHud8=', -- Public Payment Key
    '125000', -- funds
    'http://ideascale.com/t/UM5UZBf4f', -- url
    '', -- files_url
    133, -- impact_score
    '{"solution": "Build a PoC game and continue development until it''s deployed to production. Based on that experience build a game platform."}', -- extra
    'Aljosa Mohorovic', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '20 years of software development, building products for the web.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    213,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Decentralized Event-Risk Contracts',  -- title
    'Insurance is one of the oldest financial products. Hedging against risk is fundamental to life and business, yet it is a overly complicated.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    't+OHBqTglR9x1zA38N61NVEO4GJNUV2bbQQomdbcbSw=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBf4o', -- url
    '', -- files_url
    100, -- impact_score
    '{"solution": "Make an insurance marketplace where insurers and policyholders enter into contracts, initially based on on-chain data with Oracles ASAP."}', -- extra
    'Alexander Watanabe', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Worked in R&D for past 8 years:  
Problem identification  
Rapid prototyping  
3 years ago I moved into software:  
Train and deploy AI models', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    214,  -- id
    (SELECT row_id FROM objective WHERE id=5 AND event=4), -- objective
    'Cardano Hackathon',  -- title
    'Exposing outside developers and innovators to Cardano through a hackathon would bring fresh insight to the current ecosystem.',  -- summary
    (SELECT category FROM objective WHERE id=5 AND event=4), -- category - VITSS Compat ONLY
    'ZsSavFxGWiS+nhNqbKmWto3dDgjlEMITOJdh5XPq6fM=', -- Public Payment Key
    '30000', -- funds
    'http://ideascale.com/t/UM5UZBf4q', -- url
    '', -- files_url
    133, -- impact_score
    '{"solution": "Hosting a Cardano based hackathon would encourage developers to explore and experiment with the space."}', -- extra
    'wang.jon.w', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Our team has previously organized hackathons.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    215,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'imToken wallet integration',  -- title
    'Cardano needs more mobile wallets in the Asian market.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'fzI2PbffyHNp3SbZCjqpCHxHQSFQff/bKdGUF2LJOX4=', -- Public Payment Key
    '200000', -- funds
    'http://ideascale.com/t/UM5UZBf4r', -- url
    '', -- files_url
    140, -- impact_score
    '{"solution": "imToken is a mobile wallet that can support Cardano, with a huge user base in the Asian markets, with focus in China."}', -- extra
    'Philipp', -- proposer name
    '', -- proposer contact
    'https://token.im/', -- proposer URL
    '1 million monthly active users on iOS and Android. Founded in 2016, headquarters in Singapore, Hangzhou, Taiwan.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    216,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Escambo',  -- title
    'Aumentar a renda das famílias africanas.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'CFdHDQvJPlv+bvk66GDzCVH+ULAiB/6AqjoSdleRrac=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBf4u', -- url
    '', -- files_url
    131, -- impact_score
    '{"solution": "Criar a Rede Escambo para troca de servi\u00e7os e produtos entre os participantes cadastrados. Os valores s\u00e3o baseados em ADA."}', -- extra
    'Dalmar Santos', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Trabalho com famílias Africanas para a melhoria do orçamento e garantia da sustentabilidade e segurança familiar.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    217,  -- id
    (SELECT row_id FROM objective WHERE id=4 AND event=4), -- objective
    'Behavioural based governance',  -- title
    'Our governance systems are super complex and are often underdesigned, most solutions solve problems that were not fully understood',  -- summary
    (SELECT category FROM objective WHERE id=4 AND event=4), -- category - VITSS Compat ONLY
    'HNwvaqGvl38dMa6kgGn/K03XIpcW/FFb3IBQNXasGbo=', -- Public Payment Key
    '2000', -- funds
    'http://ideascale.com/t/UM5UZBf4x', -- url
    '', -- files_url
    118, -- impact_score
    '{"solution": "Design a system by understanding the problem deeply, iterating on that , and going from there."}', -- extra
    'timothy eichler', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '\+ 4 years Programming

\+ 2 year Behavioural psychology', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    218,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Cardano Hackathon',  -- title
    'Exposing outside developers and innovators to Cardano through a hackathon would bring fresh insight to the current ecosystem.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'giBqNYHS5vMY6HCcRkfPIjQL6hZeezQnuNYJJqbz+po=', -- Public Payment Key
    '30000', -- funds
    'http://ideascale.com/t/UM5UZBf4z', -- url
    '', -- files_url
    183, -- impact_score
    '{"solution": "Hosting a Cardano based hackathon would encourage developers to explore and experiment with the space."}', -- extra
    'wang.jon.w', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Our team has previously organized hackathons.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    219,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=4), -- objective
    'Self Governing Stake Pools',  -- title
    'How can we empower delegators through self governance to influence stake pool engagement and foster a vibrant and diverse pool community?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=4), -- category - VITSS Compat ONLY
    'M3NGFoK+JpKMc0du1+E+nPVSCwcSeg6tqeKVHIb+06o=', -- Public Payment Key
    '200000', -- funds
    'http://ideascale.com/t/UM5UZBf42', -- url
    '', -- files_url
    373, -- impact_score
    '{"brief": "What if stake pools were managed like a DAO (Decentralized autonomous organization)? Every decision made in a stake pool would not be centralized but voted on by the delegators. Stake pools could leverage Cardano''s governance system to promote novel and creative ways to manage pools. This would provide decentralization all the way down and empower delegators to influence the direction of their pools in a diverse and equitable way.  \r\n\r\nPool Governance Tokenomics\r\n\r\n*   Governance tokens allow token holders to govern a stake pool''s operation in a decentralized way\r\n*   Governance token give voting power to delegators to decide on proposals (see Governance Forum)\r\n*   Chosen tokenomics strategies would set the incentive structure (fix-supply, inflationary scheme, reward structure, etc.)\r\n*   Governance tokens are allocated based on staked amount but also on other parameters like longevity of stake (incentivizing loyalty)  \r\n    \r\n\r\nPool Governance Forum  \r\n\r\n*   Governance forum provide a space where ideas and proposals can be pitched and discussed by the pool community (similar to Catalyst by at the pool level)\r\n*   The forum allows delegators to discuss ideas on how to better govern and operate the pool (ie protocol parameters, validator node security and maintenance, charity donation or community projects)\r\n*   To avoid proposal spamming, a minimum staking amount would be required to submit a proposal  \r\n    \r\n\r\nPool Voting Platform  \r\n\r\n*   Voting platform provides a secure and decentralized way for a staking pool to make decisions and implement changes based on community consensus\r\n*   multi-signature smart contracts make the operation of the pool trustless and secured by the blockchain (as opposed to trusting a single Pool operator)\r\n*   Protocol changes will have a time-lock delay to allow delegators to either prepare for the changes or leave the pool  \r\n    \r\n\r\nPool Treasury  \r\n\r\n*   A portion of the pool rewards would be deposited in a Pool treasury (community owned)\r\n*   Spending from the treasury would be approved through consensus on the Voting Platform\r\n*   For Example rewards distributed to operator, marketing, charity, community projects, etc.  \r\n      \r\n    \r\n\r\nBy embracing the decentralization ethos fully, help us make the Cardano community the most empowered, engaged, vibrant and diverse community in the world!", "importance": "A stake pool''s management is centralized which hinders delegator engagement, requires trust and lacks accountability towards the community", "goal": "Pools becoming decentralized autonomous org. where the power is in the hands of the delegators and decisions are made by vote consensus", "metrics": "This challenge is looking for projects that can design generic stake pool governance frameworks to help other pools transition to a decentralized management (if they wish to). The projects must addressed one or more of the following components:\r\n\r\n*   Pool Governance Tokenomics\r\n*   Pool Governance Forum\r\n*   Pool Voting Platform\r\n*   Pool Treasury  \r\n    \r\n\r\nSuccessful proposal will also need to address incentive structures and potential risks associated with self governance."}', -- extra
    'Marvin Bertin', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    220,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=4), -- objective
    'Global Impact: Fighting COVID',  -- title
    'How can solutions built on Cardano help to deal with the COVID crisis?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=4), -- category - VITSS Compat ONLY
    'aW4UpxXbp0w+aMjzYq+33y9FaqD71KqJg3595a0PqiM=', -- Public Payment Key
    '100000', -- funds
    'http://ideascale.com/t/UM5UZBf43', -- url
    '', -- files_url
    169, -- impact_score
    '{"brief": "The world has pressing problems to solve! How can we have in impact? This challenge asks you to think about utilizing Cardano to solve problems induced by the COVID pandemic.\r\n\r\nBlockchains shine when it comes to enabling multiple parties to integrate with each other in a decentralized way. As COVID is a global problem affecting almost every part of our life, Cardano might be a good integration point.\r\n\r\nHow can solutions built on Cardano help to deal with COVID?", "importance": "\\* Help solving a top priority global problem.\r\n\r\n\\* Demonstrate Cardano''s utility", "goal": "Solutions built on / integrating Cardano are used in real world products to deal with COVID.", "metrics": "\\* No. of media articles about Cardano COVID projects\r\n\r\n\\* No. of Cardano COVID projects\r\n\r\n\\* No. of asset transfers related to wallets of those projects"}', -- extra
    'Pascal', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    221,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Funding Tree Planting Organisations',  -- title
    'Tons of Trees are being cut everyday for human use. And only a small percentage of those are being replanted.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'CQedDl8i9Nz8K0XwpZI9el74SXfpPkZRpouRan6mFMA=', -- Public Payment Key
    '100000', -- funds
    'http://ideascale.com/t/UM5UZBf44', -- url
    '', -- files_url
    144, -- impact_score
    '{"solution": "Develop a Stable Income for Green Organisations through the development of a Stakable Token (NFT) from which rewards would be spread across."}', -- extra
    'Zhan Stoyanov', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I don''t have much experience developing a token. But i''ve donated and participated in Local Events related to making the world Greener.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    222,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Better Security using Shamir Shares',  -- title
    'Securing ADA should not have loss and theft risks. However all of the Hardware wallets have single point of failure today with PK storage.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    '80FhGSlTM2GfURP8OIgUXg0MuYJI7GGEDz1LESNm/Ro=', -- Public Payment Key
    '100000', -- funds
    'http://ideascale.com/t/UM5UZBf47', -- url
    '', -- files_url
    100, -- impact_score
    '{"solution": "Integration with Cypherock X1 that uses Shamir Secret Sharing to store keys without a single point of failure on tamper proof smartcards."}', -- extra
    'Rohan Agarwal', -- proposer name
    '', -- proposer contact
    'https://cypherock.com', -- proposer URL
    'Product already has support for BTC, ETH, ERC20 tokens. It is currently in private beta with 400+ on the waitlist.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    223,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=4), -- objective
    'Celebrities Onboarding',  -- title
    'How can we make Celebs (TV, music, film) embrace Cardano as platform for a token offering this year?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=4), -- category - VITSS Compat ONLY
    'RNC3aTOUM6/m7T2H8Y1tnLMmdcIbxvnAO0FWZaEqY38=', -- Public Payment Key
    '100000', -- funds
    'http://ideascale.com/t/UM5UZBf5C', -- url
    '', -- files_url
    184, -- impact_score
    '{"brief": "To make Cardano as an innovation diffuse faster trough markets, we have to show observable, tangible results. As people look up to Celebs, embracing that target group likely leads to more observers.\r\n\r\nThe Cardano ecosystem needs to provide Celebs a high relative advantage compared to alternatives. Celebs should be provided with an easy way to get started and integrate Cardano in their portfolio.\r\n\r\nWhat tools and documents can we create for Celebs to become excited about boarding Cardano?", "importance": "1\\. More native celeb assets on Cardano -> faster diffusion of innovation -> more success stories -> repeat\r\n\r\n2\\. +assets -> +trans. -> +ADA", "goal": "A high user experience in a targeted set of tools and documents motivates Celebs to create and manage their own tokens on Cardano.", "metrics": "\\* No. of onboarded celebs\r\n\r\n\\* No. of native celeb assets created\r\n\r\n\\* No. of transactions of celeb''s tokens"}', -- extra
    'Pascal', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    224,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=4), -- objective
    'Pyramid Rewards system',  -- title
    'I would like to propose the concept of a pyramid structure in which people are actively rewarded for bringing in new people.',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=4), -- category - VITSS Compat ONLY
    'xzCOdX67LrCK6mH4m+uhP3QBkYFA59AkOs2XmpjaNyc=', -- Public Payment Key
    '2000', -- funds
    'http://ideascale.com/t/UM5UZBf5I', -- url
    '', -- files_url
    136, -- impact_score
    '{"brief": "Catalyzing the people", "importance": "Because, looking from a self interested rational perspective people need to be incentivized to bring new people in on a more active level.", "goal": "Succes would look like a system in which people can claim rewards based on the amount of new people they have introduced to the space.", "metrics": "Amount of people referred by a person"}', -- extra
    'ewmsteinebach', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    225,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Cardano School: Education Platform',  -- title
    'Introductory dev materials are scattered across the internet what makes it hard to get started with Cardano.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    '3R/vVR4f73+XCOQs3gma0C9fFSI5lTQ8UDbgIwj2Ky4=', -- Public Payment Key
    '7670', -- funds
    'http://ideascale.com/t/UM5UZBf5L', -- url
    '', -- files_url
    378, -- impact_score
    '{"solution": "I want to create a central entry point for devs to access high quality, hands-on community content teaching about Cardano. It''s like Udemy."}', -- extra
    'Pascal', -- proposer name
    '', -- proposer contact
    'https://cardano.school', -- proposer URL
    'Leading a small digitalization agency that helps well known brands e.g. by conceptualizing and providing eCommerce solutions, SEM etc.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    226,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=4), -- objective
    'Lobbying team',  -- title
    'In terms of spreading Cardano not all people are created equal. How can we reach the people that reach masses?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=4), -- category - VITSS Compat ONLY
    'DaRj+NG7btvPN0Rywyj13v2UhB/01+ah9Vf5T4+zQO8=', -- Public Payment Key
    '2000', -- funds
    'http://ideascale.com/t/UM5UZBf5Q', -- url
    '', -- files_url
    175, -- impact_score
    '{"brief": "Can we create a team that focusses solely on lobbying Cardano to highly influencial people?", "importance": "Because a focus on institutional involvevement, involvement of celebrities etc. will lead others to bridge the gap to accepting Cardano.", "goal": "A dedicated lobbying team with connections to think tanks and governments, tech pioneers, and corporations around the world.", "metrics": "none"}', -- extra
    'ewmsteinebach', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    227,  -- id
    (SELECT row_id FROM objective WHERE id=4 AND event=4), -- objective
    'Catalystwinners.com',  -- title
    'It is currently not easy to find which projects have won on each Fund. That could eventually undermine the transparency of Catalyst.',  -- summary
    (SELECT category FROM objective WHERE id=4 AND event=4), -- category - VITSS Compat ONLY
    'WgxjgDKKyew/Gpq1rKuFPtEkuPPOsrZ+zjxqWsyEYDE=', -- Public Payment Key
    '5680', -- funds
    'http://ideascale.com/t/UM5UZBf5X', -- url
    '', -- files_url
    313, -- impact_score
    '{"solution": "A website showing the winning projects of each Fund and Campaign. Making Catalyst more transparent. This would encourage more participation."}', -- extra
    'Ryan Morrison', -- proposer name
    '', -- proposer contact
    'http://catalystwinners.com', -- proposer URL
    'I''ve built CardaNews.com and I run the Cardano Podcast.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    228,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Art DApps to fund & curate artwork',  -- title
    'No. 1 problem of the art industry for the past 3 years is onboarding new collectors *(tech savvy, gadget-driven* *millennials)*',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'ydeUUJpa4z5kkdn6wbtWinJXL99phG9i/A8mnjDZrFQ=', -- Public Payment Key
    '22567', -- funds
    'http://ideascale.com/t/UM5UZBf5k', -- url
    '', -- files_url
    419, -- impact_score
    '{"solution": "Art DApps Platform, inspired by IdeaScale format with focus on creation and curation of arts collaborating with art-related Fund winners"}', -- extra
    'Yan Tirta', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '*   Yan Tirta, Marcomm Manager of benda Art Management
*   Ignatia Nilu, Curator of ArtJog and ArtBali
*   Teguh Harmanda, COO of Tokocrypto', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    229,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=4), -- objective
    'Allow the vote for network updates',  -- title
    'Updates are essential. It is going to be important to think about an updating process with community votes and discussions',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=4), -- category - VITSS Compat ONLY
    'Pylq+NyrPMBmK2uTQ817BVvYB2ldnk1zKM/sQou7Tms=', -- Public Payment Key
    '12000', -- funds
    'http://ideascale.com/t/UM5UZBf5l', -- url
    '', -- files_url
    214, -- impact_score
    '{"brief": "Cardano is a new protocol and it''s community is going to expand. Involving ADA''s holders in Cardano updates is a challenge and a necessity. We propose to reflect on a validation process, \"onchain\", for the updates (as other protocols allow) and to propose it to the community.", "importance": "The governance and resilience of a protocol requires the agreement of the majority of all it''s members when amendments are made.", "goal": "Make Cardano the best self-amendable protocol and build the voting process based on the vote in Catalyst (take inspiration from Dash/Tezos)", "metrics": "The participation rate in amendment proposals is the key indicator. It is also necessary to think about different validation steps from the submission of an update to it''s implementation."}', -- extra
    'Balbublock', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    230,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Afri-pay',  -- title
    'Africans often do not use bank accounts and even when they do they have the control they should when transferring money from A to B.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    '/T+lXolry48xvAMUAtmOHAz8AX/lRzEdmMusm/J3+uU=', -- Public Payment Key
    '3000', -- funds
    'http://ideascale.com/t/UM5UZBf5m', -- url
    '', -- files_url
    267, -- impact_score
    '{"solution": "Design of a money transfer platform using Blockchain technology, allowing users to transfer money from one account to another without limit."}', -- extra
    'uptodatedevelopers', -- proposer name
    '', -- proposer contact
    'https://github.com/UPTODATE-DEV', -- proposer URL
    'Highly enthusiastic and engaged group of local developers, WADA network technical support', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    231,  -- id
    (SELECT row_id FROM objective WHERE id=5 AND event=4), -- objective
    'Telecom infrastructure tokenization',  -- title
    'Build a team to help the tokenization of a several open RAN municipal projects.',  -- summary
    (SELECT category FROM objective WHERE id=5 AND event=4), -- category - VITSS Compat ONLY
    'MNxLPxSEyY7k1BOTvnfpjrDcAjoqhBkMXz3nH/pqH8Q=', -- Public Payment Key
    '30000', -- funds
    'http://ideascale.com/t/UM5UZBf52', -- url
    '', -- files_url
    118, -- impact_score
    '{"solution": "Enabling the people and municipalities to own the telecommunications network infrastructure where operators offer their service"}', -- extra
    'rui cunha', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'programers, influencers and any one that feels could help.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    232,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'CardaWork.com - Work Marketplace',  -- title
    'It is difficult for developers to get excited about Cardano if they cannot monetize their Cardano development skills (Plutus, Marlowe, Glow)',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    '6sNt0k5fiuMaaudVwsMwp1M8t0gmcnMtidD7lfzRk50=', -- Public Payment Key
    '19700', -- funds
    'http://ideascale.com/t/UM5UZBf55', -- url
    '', -- files_url
    213, -- impact_score
    '{"solution": "A marketplace where developers can show their Cardano programming skills and people/companies can hire them for projects."}', -- extra
    'Ryan Morrison', -- proposer name
    '', -- proposer contact
    'http://cardawork.com', -- proposer URL
    '13 years experience in digital marketing. I''ve created the Cardano Podcast and CardaNews. Commited to the growth of the Cardano community.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    233,  -- id
    (SELECT row_id FROM objective WHERE id=7 AND event=4), -- objective
    'L10N - localisation',  -- title
    'community + communication should start in the low cost high value - online - (through the best medium - video) then offline.',  -- summary
    (SELECT category FROM objective WHERE id=7 AND event=4), -- category - VITSS Compat ONLY
    'WVZ6NnBUrhyPTx+b+hVOpjaIZF6P3ScFox7jFTcqr94=', -- Public Payment Key
    '10000', -- funds
    'http://ideascale.com/t/UM5UZBf6E', -- url
    '', -- files_url
    107, -- impact_score
    '{"solution": "utilise video (L10N program on caaast.live) to solve the high level communication, collaboration and compensation."}', -- extra
    'A', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    234,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Geographic centers of expertise',  -- title
    'In order to build partnerships and move forward on specific jurisdictions, country-specific ancillary entities are good answer',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    '6MKh/3Pzco5hiMjFg2Css1a2bWT2JKIcsJ765v7i8OY=', -- Public Payment Key
    '42000', -- funds
    'http://ideascale.com/t/UM5UZBf6F', -- url
    '', -- files_url
    233, -- impact_score
    '{"solution": "Cardano needs concrete use cases. Support teams that know the local issues and have access to partners as well, train and assist developers"}', -- extra
    'Balbublock', -- proposer name
    '', -- proposer contact
    'https://smart-chain.fr/', -- proposer URL
    'We are a French team of 15 people (researcher-developer) and have developed many use cases and partnerships for 3 years on Eth and Tezos.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    235,  -- id
    (SELECT row_id FROM objective WHERE id=7 AND event=4), -- objective
    'LCC in Brazil for Growth and Adopt.',  -- title
    'Lack of optimization in community communication during Cardano Growth Phase, we need more content translation and core members are overload.',  -- summary
    (SELECT category FROM objective WHERE id=7 AND event=4), -- category - VITSS Compat ONLY
    'XDhT5MFC5pndCb1gwwHCkCi2Ifmz/UT+GbwPGzdeKTw=', -- Public Payment Key
    '14400', -- funds
    'http://ideascale.com/t/UM5UZBf6O', -- url
    '', -- files_url
    385, -- impact_score
    '{"solution": "We want to hire more content translators, designers and maybe video editors. We need to strength our presence in Brazil and Lusitans."}', -- extra
    'Thiago Nunes', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'IT background, entrepreneur and relevant experience in Marketing and Digital Strategies segment. Currently Cardano Ambassador Lvl. 3 and SPO', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    236,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Local support hub to assit devs',  -- title
    'Local support is often necessary to train and support developers and community, address specific issues and create ambitious projects',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'lr3EOI4XLIyoeB/04Q7bK3uvHFKDkKgCpl5cqEhvjoA=', -- Public Payment Key
    '25000', -- funds
    'http://ideascale.com/t/UM5UZBf6c', -- url
    '', -- files_url
    224, -- impact_score
    '{"solution": "We propose to be one of the first support hubs, to gather best practices and bring concrete projects and several developers on Cardano"}', -- extra
    'Balbublock', -- proposer name
    '', -- proposer contact
    'https://smart-chain.fr/', -- proposer URL
    'We are 14 people (developer - researcher) working on many projects in the French ecosystem on different Blockchain (Ethereum & Tezos)', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    237,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Universal news subscription',  -- title
    'News organisations rely heavily on ads that rewards clicks which result in poor quality content.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'il+s9NIMqvd9L0uqkWkDYjCtuACSu1SDSyeOOlBUjRM=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBf6j', -- url
    '', -- files_url
    294, -- impact_score
    '{"solution": "Introduce a DApp with associated token. Users purchase token and gets ad-free access to participating news sites. DApp pay sites accordingly"}', -- extra
    'plcplc', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I have 7 years of professional experience as a software developer. 4 of those are with Haskell. Also MSc. in Computer Science.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    238,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Nodeless command line interface',  -- title
    'You need to run a full Cardano node to interact with the blockchain information using command line interface.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    '1uCmxJuCM3cCXKLr/9gmtd9+epyAaOEo7v0Zbc0mkLQ=', -- Public Payment Key
    '14000', -- funds
    'http://ideascale.com/t/UM5UZBf68', -- url
    '', -- files_url
    400, -- impact_score
    '{"solution": "We would like to build a lightweight command line interface on top of Blockfrost, API as a Service, saving developers'' time and resources."}', -- extra
    'Five Binaries', -- proposer name
    '', -- proposer contact
    'https://blockfrost.io/', -- proposer URL
    'We are the creators of Blockfrost.io. Creating services and tools lowering barriers to entry for developers is our bread and butter.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    239,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'SEPA to/from Cardano addresse',  -- title
    'On-boarding users is not easy. This service would provide a easy way for EU citizen to send/receive € on/from Cardano',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'qPDQDy06sUgfixrNdigG/Or4ldwpxVy2nCG2zJmDq+M=', -- Public Payment Key
    '5000', -- funds
    'http://ideascale.com/t/UM5UZBf7B', -- url
    '', -- files_url
    193, -- impact_score
    '{"solution": "A bank account managed with weboob + tokens created/redeemed on the ledger."}', -- extra
    'Régis GUYOMARCH', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Fullstack developer, I''ve already created this kind of service with Stellar for a non-profit organization. It was only for internal use.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    240,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Updev Community et Cardano Catalyst',  -- title
    'Even though we have a strong local community of devs, they have limited skills in blockchain tech and no way to access the information.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'QoVrcwr+uiD/0fClViZR6JwiNSELLfkPLQlDkn4N/24=', -- Public Payment Key
    '4000', -- funds
    'http://ideascale.com/t/UM5UZBf7F', -- url
    '', -- files_url
    380, -- impact_score
    '{"solution": "Host regular meet ups with blockchain developers from WADA''s network to engage the community and erase the barriers around blockchain tech."}', -- extra
    'uptodatedevelopers', -- proposer name
    '', -- proposer contact
    'https://github.com/UPTODATE-DEV', -- proposer URL
    'Highly enthusiastic and engaged group of local developers, WADA network educational support', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    241,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Smart Contracts Tutorials',  -- title
    'Developpers not used to working in Haskell and/or with blockchain tech find it hard to introduce themselves to smart contract development',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'jnnIRGO/34cf6e1qq82O+dbPIrbY5PTsvfh22jPTRME=', -- Public Payment Key
    '5003', -- funds
    'http://ideascale.com/t/UM5UZBf7I', -- url
    '', -- files_url
    189, -- impact_score
    '{"solution": "Explain from the ground up the various mental models and tools required to understand, create and execute smart contracts"}', -- extra
    'Marc-André Brochu', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I am a professional developper with a degree in maths and "on the side" experience with Haskell, smart contracts and blockchain tech', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    242,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'The Universal Cookbook',  -- title
    'What are we making for dinner tonight?',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'bb37J4OPFL2K/Ma00y1TQ02/mc6k/8eJ8JTry9zZ0Qg=', -- Public Payment Key
    '5003', -- funds
    'http://ideascale.com/t/UM5UZBf7R', -- url
    '', -- files_url
    167, -- impact_score
    '{"solution": "Just take a look at the Universal Cookbook, where recipes from all over the world await"}', -- extra
    'Marc-André Brochu', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I am a professional developper with webdev, Haskell and smart contracts experience who likes to cook', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    243,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=4), -- objective
    'More wood behind fewer arrows',  -- title
    'How can we increase Catalyst proposal strength?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=4), -- category - VITSS Compat ONLY
    'Y4uEfC6wb8O6NLKREGgTPdnTqjjQmVM26yQseXn2GWM=', -- Public Payment Key
    '250000', -- funds
    'http://ideascale.com/t/UM5UZBf7V', -- url
    '', -- files_url
    180, -- impact_score
    '{"brief": "My Challenge title at the top, and this proposal, is ''meta.''\r\n\r\n  \r\n\r\n\"More wood behind fewer arrows\" refers to the idea of reducing the number of challenges to 1 (because \"prioritization is only working when it hurts\" :) I don''t really mind what the challenge is, as long as there is only one.\r\n\r\n  \r\n\r\nBut since there were already many good ones, instead of setting new challenges, I propose to double down on the previously under-engaged challenges.  \r\n  \r\n\r\nMy specific proposal for the previous challenge to double down on is:\r\n\r\n  \r\n\r\n\\*F4: Proposer Outreach\\*\r\n\r\n  \r\n\r\nSince these funds are sequential, they can be organized recursively, and a single challenge to develop and execute more proposer outreach will produce a positive feedback loop for the next one.", "importance": "I saw many good challenges set, but they did not get enough strong proposals. Attention needs to be focused! F3 set 3 challenges, F4 had 6\u2026", "goal": "F3 and F4 Dapps challenges got over 100 proposals each, while most others got 1-3 dozen. A single F6 challenge will be successful with 200+.", "metrics": "By reducing the number of challenges to 1, I expect all the other metrics of engagement on the ideascale platform - number of proposals, number of votes, kudos, and comments on those proposals - to increase, in aggregate; and the IOHK team will see the perceived quality of the top voted proposals improve."}', -- extra
    'Dave Crossland', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    244,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'ADA human resources',  -- title
    'job applicants want to improve their resume or if they don''t, they want to make a resume to improve their job offer opportunities.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    '17A3vhQg33bVmW4qxTHYyyrxBr/70UHQ6K/YPSxdCJI=', -- Public Payment Key
    '4000', -- funds
    'http://ideascale.com/t/UM5UZBf7b', -- url
    '', -- files_url
    135, -- impact_score
    '{"solution": "the applicant sends his resume to be reviewed and a human resources professional reviews it."}', -- extra
    'Hugo Ojeda', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'software engineer', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    245,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'ADA Powers Incentive Apps',  -- title
    'Easy GTM strategy for ADA to drive wider adoption and liquidity.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'qW9IWYtR4gAkIJhSEmu1LiNE7Ov54rKPrX2aiGehqaU=', -- Public Payment Key
    '35000', -- funds
    'http://ideascale.com/t/UM5UZBf7j', -- url
    '', -- files_url
    108, -- impact_score
    '{"solution": "Drive ADA adoption by powering loyalty, gift card, and incentive programs by building a simple API for these apps."}', -- extra
    'John Arbour', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'We are a team of long time software developers (4 companies and 2 exits) and business professionals.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    246,  -- id
    (SELECT row_id FROM objective WHERE id=4 AND event=4), -- objective
    'Open Source Governance UI',  -- title
    'Users are overwhelmed by distributed decision making, and developers do not have the funding to complete the network upgrades.',  -- summary
    (SELECT category FROM objective WHERE id=4 AND event=4), -- category - VITSS Compat ONLY
    'eRNsNncBhSbY0w6KZ+LElHIH7E719bSA0xv1Slfylus=', -- Public Payment Key
    '25000', -- funds
    'http://ideascale.com/t/UM5UZBf7u', -- url
    '', -- files_url
    343, -- impact_score
    '{"solution": "CommonWealth provides blockchain networks a reliable and time-tested UX/UI to empower their users and devs to make decisions quickly."}', -- extra
    'Nathan Windsor', -- proposer name
    '', -- proposer contact
    'https://commonwealth.im/', -- proposer URL
    'The team at CommonWealth and Macroscape have worked on projects for the ETH foundation, Edgeware, and Nodle. We are collaborating for this.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    247,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'PoolPeek.com Upgrades',  -- title
    'PoolPeek.com is built by small pool owners for small pool delegators. Small stake pools need more ways for delegators to find their pools.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    '2mhk8JSDLtNWfZ39v7Rs4MWyeTcEdWNbO3rdsX+Mjl4=', -- Public Payment Key
    '20000', -- funds
    'http://ideascale.com/t/UM5UZBf7z', -- url
    '', -- files_url
    200, -- impact_score
    '{"solution": "offer additional views where delegators can see small pools in interesting ways, add a chatbot, create a new cardano token viewer website."}', -- extra
    'TRAIN', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Paddy: 11 years development experience and a degree in game development

Craig: 25 years development experience and a BS in Computer Science', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    248,  -- id
    (SELECT row_id FROM objective WHERE id=5 AND event=4), -- objective
    'Adaption and integration of Cardano',  -- title
    'Implementing Cardano in society requires also many non ICT technical aspects: legal, political, stakeholders postions, etc',  -- summary
    (SELECT category FROM objective WHERE id=5 AND event=4), -- category - VITSS Compat ONLY
    'l+nlVE55DP16bDXx3T1qTbEAZxhow15RH7iQXlAR1C4=', -- Public Payment Key
    '10000', -- funds
    'http://ideascale.com/t/UM5UZBf76', -- url
    '', -- files_url
    208, -- impact_score
    '{"solution": "By use of an interdisciplinary scientific approach and a powerful network of strategist, political scientists."}', -- extra
    'kessen', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Strategic legal, political advisor in many governmental institutions and research organizations.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    249,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Genomic Data Marketplace',  -- title
    'Current markets for genomic data are centralized and run by the two major genetic testing companies. Ours is a democratic "de-sci" alternate',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'UXgs0O25EsAnD8/NYc2/oDT7cHz/fy74RubYQn1eoW4=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBf8F', -- url
    '', -- files_url
    317, -- impact_score
    '{"solution": "Our Gene-Chain allows peer-to-peer, compensated and transparent transactions of genomic data where users are aware and rewarded directly."}', -- extra
    'drkoepsell', -- proposer name
    '', -- proposer contact
    'http://encrypgen.com', -- proposer URL
    'We launched the Gene-Chain using an erc20 token, $DNA in November 2018. We are now seeking to integrate with additional partners', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    250,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'REVUTO - Subscription Management ',  -- title
    'Subscription management via Revuto Virtual Debit Cards supporting Cardano. For more please click: https://bit.ly/revuto_cardano',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'lAw0c5BjdZxOL6YD2PgiPujHzof9moE/3ixuoxLU9+8=', -- Public Payment Key
    '60000', -- funds
    'http://ideascale.com/t/UM5UZBf8M', -- url
    '', -- files_url
    383, -- impact_score
    '{"solution": "Subscription management solution: allowing users to actively manage subscription seasonality and enjoy a one click subscription experience."}', -- extra
    'josipa majic', -- proposer name
    '', -- proposer contact
    'http://revuto.com/', -- proposer URL
    'CTO ex head of AI at Intuit, CEO founder and CEO of emotional analytics firm working for global banks, COO crypto investor and founder', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    251,  -- id
    (SELECT row_id FROM objective WHERE id=5 AND event=4), -- objective
    'Gauntlets of Catalyst Courts',  -- title
    'Project Catalyst wants to attract the best/brightest outside entrepreneurs/firms and submit proposals in F5/F6.',  -- summary
    (SELECT category FROM objective WHERE id=5 AND event=4), -- category - VITSS Compat ONLY
    '3FX6H9mIYe5gwirrHKnyhVL6QmEWMNj8aoy3il/cQEU=', -- Public Payment Key
    '25000', -- funds
    'http://ideascale.com/t/UM5UZBf8S', -- url
    '', -- files_url
    271, -- impact_score
    '{"solution": "Post this Challenge on LinkedIN/Social Media and ask for comments, invite people to ideascale to create profiles and review proposals."}', -- extra
    'Q U A S A R', -- proposer name
    '', -- proposer contact
    'https://www.linkedin.com/in/deryck-lance-9405898/', -- proposer URL
    'Recruiting, Placement, Procurement, Marketing, Product/Project Validation, Networking', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    252,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Haskell DeFi SDK for Plutus Devs',  -- title
    'Cardano DeFi developers need an open-source standardized method for querying several DeFi protocols in parallel to build composable L2 dApps',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    '59kt5RAaGx4I1lpigsA3zeUxEMfm5rJlOvLB6ye3UZA=', -- Public Payment Key
    '56500', -- funds
    'http://ideascale.com/t/UM5UZBf8U', -- url
    '', -- files_url
    493, -- impact_score
    '{"solution": "Plutus DeFi SDK is an open-source set of smart contracts designed for DeFi portfolio accounting; the on-chain *balanceOf* for DeFi protocols."}', -- extra
    'Liqwid Labs', -- proposer name
    '', -- proposer contact
    'https://www.mlabs.city/', -- proposer URL
    'Develops payment platforms in Haskell  
Open-source: github.com/juspay/euler-hs  
hackage.haskell.org/package/medea  
haskell-beam.github.io/beam/', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    253,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Sustainable Forest Stablecoin',  -- title
    'High gas prices on Ethereum are crippling Ekofolio''s vision of frictionless issuance and trade of Stablecoins backed by Natural Capital.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'nkfKySYJh0bts24si+YU4R01grwthGG2savUnhkBza8=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBf8g', -- url
    '', -- files_url
    338, -- impact_score
    '{"solution": "Assess feasibility and deploy Ekofolio''s native EKO and FOLIO tokens on Cardano, adding a sustainable Stablecoin use-case to the project."}', -- extra
    'Jason', -- proposer name
    '', -- proposer contact
    'https://www.ekofolio.com/', -- proposer URL
    'Ekofolio founded 2017. Proof of concept completed. Won grant from European Space Agency for development of real-time monitoring of forests.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    254,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'The Predictors App - Oracle Service',  -- title
    'It can be hard to find reliable predictions on the prices of assets, outcome of events, and other things people are interested in knowing.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'HDS8oNF1G79yTRGm1SGi10W9Uggn8v3ejw6u8bGfyI8=', -- Public Payment Key
    '10000', -- funds
    'http://ideascale.com/t/UM5UZBf8i', -- url
    '', -- files_url
    111, -- impact_score
    '{"solution": "With The Predictors APP you can crowdsource the data you need to make better decisions, you can win prizes from making correct predictions."}', -- extra
    'Boone Bergsma', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Predictions markets are growing in popularity and crowdsourced data can provide wisdom and insights to people, businesses, organizations.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    255,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Ouroboros Networking Lib in JS',  -- title
    'Cardano has tools only in Haskell, developers are constrained due to the lack of a networking library in other languages such as Javascript',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    '36/DgftJLuA4hjcAzahID5VToQxw1bEnXNbccc5rEI4=', -- Public Payment Key
    '80000', -- funds
    'http://ideascale.com/t/UM5UZBf8n', -- url
    '', -- files_url
    464, -- impact_score
    '{"solution": "Create Ouroboros Networking package in Javascript, which allows talking to Cardano node by easily installing the NPM package"}', -- extra
    'Ashish Cardanoscan', -- proposer name
    '', -- proposer contact
    'https://cardanoscan.io', -- proposer URL
    'We have developed a part of the networking layer as part of our implementation with the Cardanoscan custom backend system', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    256,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Haskell Devs for Liqwid Plutus SC''s',  -- title
    'Plutus is Cardano''s smart contract layer written in Haskell. 1st target users to build secure DeFi contracts in Plutus are senior Haskellers',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'gZVriAjIflTzhzHLcn8R8XeojcQ0rSJydVLS8X4IpOA=', -- Public Payment Key
    '36500', -- funds
    'http://ideascale.com/t/UM5UZBf8o', -- url
    '', -- files_url
    487, -- impact_score
    '{"solution": "Team of 6 senior fullstack Haskell devs with deep fintech backgrounds currently building on the Plutus eUTXO smart contract system on TN."}', -- extra
    'Liqwid Labs', -- proposer name
    '', -- proposer contact
    'https://www.mlabs.city/', -- proposer URL
    'Develops fintech payment platforms & works on open-source projects in Haskell.  
juspay.in/ Migrated business logic from Node.JS to Haskell.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    257,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Cardax - DEX on Cardano',  -- title
    'There is currently no decentralized exchange (DEX) on Cardano. Tokens built on Cardano dont have a ''native exchange'' to list yet.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'p8WnmoNAHnb13UeMYaB8Yz1OgKni8yL7r6kJJHovK1M=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBf8r', -- url
    '', -- files_url
    261, -- impact_score
    '{"solution": "Cardax - The first DEX in the Cardano ecosystem. Cardax will be like Uniswap but on Cardano."}', -- extra
    'Ryan Morrison', -- proposer name
    '', -- proposer contact
    'https://cardax.gitbook.io/cardax/', -- proposer URL
    'I run Quant Digital which develops on Cardano. We have both the technical and marketing knowledge in our team to make this a success.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    258,  -- id
    (SELECT row_id FROM objective WHERE id=5 AND event=4), -- objective
    'Beautiful and clear results page(s)',  -- title
    'As a person new to catalyst, I want to know the results/status of previous Funds.

  

The F1 PDF was a good start, but is now out of date.',  -- summary
    (SELECT category FROM objective WHERE id=5 AND event=4), -- category - VITSS Compat ONLY
    '2Xqd4h548e6tN7TXoMy8p96esvNZyileR6SQBugPhv4=', -- Public Payment Key
    '36000', -- funds
    'http://ideascale.com/t/UM5UZBf8v', -- url
    '', -- files_url
    200, -- impact_score
    '{"solution": "IdeaScale is a good platform for starting up a fund, but funded proposals should live in a Cardano Foundation space."}', -- extra
    'Dave Crossland', -- proposer name
    '', -- proposer contact
    'https://github.com/cardano-foundation/catalyst', -- proposer URL
    'I started the http://designwithfontforge.com project, a Github Pages website to provide an open textbook for the FontForge font editor.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    259,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Proof-of-Free-Will ADA HW wallet',  -- title
    'Current solutions for protecting digital assets are missing the tight bond to real physical identity of a person beyond simple PIN/Password.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'whbXG7gJHDR3lXCv3eqIBQ69MQKAhEeTP7B+0aOqZLA=', -- Public Payment Key
    '80000', -- funds
    'http://ideascale.com/t/UM5UZBf9M', -- url
    '', -- files_url
    314, -- impact_score
    '{"solution": "ADA first HW wallet capable of protecting keys by static & behavioral biometrics to enable secure transactions without passwords."}', -- extra
    'Peter', -- proposer name
    '', -- proposer contact
    'https://www.crayonic.com', -- proposer URL
    'We are developers of biometric MFA smart auth. token Crayonic KeyVault with multiple crypto HW for blockchain projects accomplished to date.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    260,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Deep Data Management Solution',  -- title
    'Blockchain state management has become completely unmanageable. As Cardano grows, the database has become fast to write to but slow to read.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'zo5wa+xulO3C8WFLKsQS968j2wRjvpmbHRH+IQ+l8ow=', -- Public Payment Key
    '75000', -- funds
    'http://ideascale.com/t/UM5UZBf9S', -- url
    '', -- files_url
    157, -- impact_score
    '{"solution": "We have built a custom indexer with a custom data compression algorithm that helps cache huge amounts of blockchain data for ease of access."}', -- extra
    'Nathan Windsor', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'We have an exclusive license to a custom database, knoxdb, which is used as an indexer for other blockchains as Bitcoin, Doge, and Flow.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    261,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Distributed decision making Dapp',  -- title
    'There are no tools to assist decision making already built on Cardano which could expand outside the community as well',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    '5V4XipX6l31IDkmhVouDXH+U4dD7GfvWoPRHu3caYiU=', -- Public Payment Key
    '40800', -- funds
    'http://ideascale.com/t/UM5UZBf9W', -- url
    '', -- files_url
    250, -- impact_score
    '{"solution": "Creating open-source governance tools and proving documentation\r\n\r\nA permanent mark on Cardano blockchain to share the plans and decisions"}', -- extra
    'Tevo', -- proposer name
    '', -- proposer contact
    'https://miro.com/app/board/o9J_lRA_fCw=/', -- proposer URL
    'Created voting dapps w/ Solidity, Hyperledger, and Solana tools.

Team Leader, Project Manager, Front-end Developer, 2 Back-end Developers', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    262,  -- id
    (SELECT row_id FROM objective WHERE id=4 AND event=4), -- objective
    'Governance tools to boost collabs',  -- title
    'Information about The Project Catalyst is not well structured and easily accessible',  -- summary
    (SELECT category FROM objective WHERE id=4 AND event=4), -- category - VITSS Compat ONLY
    'YJaX1nlf6XF5+PSHtlX8aK+4qtXK5JWAhz3ABkpI4zA=', -- Public Payment Key
    '40320', -- funds
    'http://ideascale.com/t/UM5UZBf9Y', -- url
    '', -- files_url
    207, -- impact_score
    '{"solution": "Creating tools for gathering information from the community\r\n\r\nInformation stored in Cardano network\r\n\r\nData used for reports and insight"}', -- extra
    'Tevo', -- proposer name
    '', -- proposer contact
    'https://miro.com/app/board/o9J_lRA_fCw=/', -- proposer URL
    'Created voting dapps w/ Solidity, Hyperledger, and Solana tools.

Team Leader, Project Manager, Front-end Developer, 2 Back-end Developers', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    263,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'DAO for creating governance tools',  -- title
    'Voting is prone to errors and expensive

There are no helpful and resource-friendly tools to help us govern ourselves and what we do',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'c+EcaxDSErQ1qrXzHsqVT1oHsdlBB12sAQ25iK8A+aM=', -- Public Payment Key
    '87480', -- funds
    'http://ideascale.com/t/UM5UZBf9b', -- url
    '', -- files_url
    176, -- impact_score
    '{"solution": "Creating Governance tools to collaboratively manage ideas and tasks\r\n\r\nDapps connect through Cardano for built-in security and transparency"}', -- extra
    'Tevo', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Created voting dapps w/ Simbachain and Solana tools

Team Leader, Project Manager, Front-end Developer, 2 Back-end Developers, and Marketing', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    264,  -- id
    (SELECT row_id FROM objective WHERE id=7 AND event=4), -- objective
    'Fund Matching from C-Foundation',  -- title
    'Cardano Foundation is looking for ways to create a stronger impact in local areas and also reinvigorate the Ambassador program.',  -- summary
    (SELECT category FROM objective WHERE id=7 AND event=4), -- category - VITSS Compat ONLY
    'HLH9aBmy504fv1//NgJl4HinE2VgsjG6FtceIOkqkok=', -- Public Payment Key
    '30000', -- funds
    'http://ideascale.com/t/UM5UZBf9f', -- url
    '', -- files_url
    192, -- impact_score
    '{"solution": "I propose that the Cardano Foundation match funding that LCCs generate/raise/earn. Location, geography first, digital community second."}', -- extra
    'Q U A S A R', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I have worked in the digital/physical platform world that we now live in and I have been organizing events and communities for 20yrs.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    265,  -- id
    (SELECT row_id FROM objective WHERE id=7 AND event=4), -- objective
    'Strengthen local communities',  -- title
    'How to generate more active Cardano participants, that effect local change?Many don''t know how to use a Cardano wallet & need face2face help',  -- summary
    (SELECT category FROM objective WHERE id=7 AND event=4), -- category - VITSS Compat ONLY
    'ZwKuUjShTdeLgg+WDRbs9IKMaEUbpC/oPvq3TBubrjc=', -- Public Payment Key
    '1337', -- funds
    'http://ideascale.com/t/UM5UZBf9i', -- url
    '', -- files_url
    185, -- impact_score
    '{"solution": "Give away for a couple of Ada. People come, setup wallet, receive Ada and \"pay\" for product. strengthens local business & wins participants."}', -- extra
    'Dan Verowski', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Berlin offers many local spots to leverage.

Aramen loves to offer Spirum. The vision aligns well with Cardano: https://aramen.life/vision/', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    266,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Community-driven content production',  -- title
    'Producing informational/educational content involves a lot of work, and it''s hard to predict what is valuable to the Cardano community',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'slgBzwJWPHlAYVdDJANTgMifNMQ6XTVGdP7BscVtr7Y=', -- Public Payment Key
    '22200', -- funds
    'http://ideascale.com/t/UM5UZBf9n', -- url
    '', -- files_url
    357, -- impact_score
    '{"solution": "A collab platform for community-driven content production based on content models with built-in token-based governance on content and models"}', -- extra
    'Lincon Vidal', -- proposer name
    '', -- proposer contact
    'https://everyblock.studio/project/infoblocks', -- proposer URL
    'Our team has been exploring for 6 months the challenging task of educating people about Cardano''s technology through the Infoblocks project.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    267,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'ADA MyProject Freelance',  -- title
    'High demand for blockchain-related projects',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'Zra38DPmlpqmgWElD3vHV7rdFSDI4ySZvh+X5b8MJ+k=', -- Public Payment Key
    '5000', -- funds
    'http://ideascale.com/t/UM5UZBf9o', -- url
    '', -- files_url
    155, -- impact_score
    '{"solution": "Using a platform to unify the needs to carry out technology projects and hire freelancers to develop the projects"}', -- extra
    'Hugo Ojeda', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'software engenieer', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    268,  -- id
    (SELECT row_id FROM objective WHERE id=5 AND event=4), -- objective
    'Cardano Vision Website Germany',  -- title
    'The germans are very sceptic on the entire cryptospace.  
Many barely know that bitcoin exists and think it exists for criminal use only.',  -- summary
    (SELECT category FROM objective WHERE id=5 AND event=4), -- category - VITSS Compat ONLY
    '3F5QHqqs5VGnaq5tswIg75V6i3LGzEIwVU1um5RAF/8=', -- Public Payment Key
    '4500', -- funds
    'http://ideascale.com/t/UM5UZBf9w', -- url
    '', -- files_url
    290, -- impact_score
    '{"solution": "We create a website and host events that makes Cardanos vision visible for all germans.  \r\n> Educate about Catalyst and get students on board."}', -- extra
    'janekholsten', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I have background in programming and motivated studs.  
We are a group of students that study entrepreneurship and founded a company to do so.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    269,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'These Walled Gardens - DevThemePark',  -- title
    'We live/operate in physical/digital silos that limit the flow in and out of social groups and devs need a fast track lane to teams/tools.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'F1s92E2aW53/TPJAY7jITfeGFjMBPaBIxE77mSF06iU=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBf9z', -- url
    '', -- files_url
    167, -- impact_score
    '{"solution": "Create a contextual microcosmic space to onboard devs and show the way to the playground. The Garden the self-identification center to zoo."}', -- extra
    'Q U A S A R', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Independent Scholar with a focus on social networks, linguistics, and ideaflow.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    270,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Bequeath:  Inheritance tracking',  -- title
    'Assignment, tracking, claiming, distribution of personal property after death',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'TM/pchefN2vU+8yP4+kIVC/4JMkUrhVggf7SD+1Pes8=', -- Public Payment Key
    '40000', -- funds
    'http://ideascale.com/t/UM5UZBf90', -- url
    '', -- files_url
    290, -- impact_score
    '{"solution": "Living wills have been around for centuries. We need to move them to a blockchain with NFTs & added benefits of real-time data availability."}', -- extra
    'Clint Morgan', -- proposer name
    '', -- proposer contact
    'https://www.linkedin.com/in/clint-morgan-513b39/', -- proposer URL
    'I have 20 years of full-stack (MS) development experience. I''m also an Entrepreneur that would love to leave something useful behind.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    271,  -- id
    (SELECT row_id FROM objective WHERE id=6 AND event=4), -- objective
    'Social Physics-Balloon Challenge',  -- title
    'Catalyst lacks the ability to create "value" to engage community members. We are also challenged with defining "value".',  -- summary
    (SELECT category FROM objective WHERE id=6 AND event=4), -- category - VITSS Compat ONLY
    'YGesy75JNlybSQm+5knhXM1xDmt9AFJXNw6aBI4HW+U=', -- Public Payment Key
    '30000', -- funds
    'http://ideascale.com/t/UM5UZBf94', -- url
    '', -- files_url
    283, -- impact_score
    '{"solution": "If we combine the DARPA''s Red Balloon Challenge and create agile teams, we incentivize the community members, proposers, mentors, advisors."}', -- extra
    'Q U A S A R', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Independent Academic with a focus on social behavior, social networks, a collective intelligence.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    272,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=4), -- objective
    'Build user-centric apps on Cardano',  -- title
    'There is a steep learning curve to implement user-centric applications that support privacy and verifiable evidence on Cardano.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=4), -- category - VITSS Compat ONLY
    'cleXUd24hAiX9wyY93qzUD724KgzjhTaMCj/NZnIxn8=', -- Public Payment Key
    '29000', -- funds
    'http://ideascale.com/t/UM5UZBf95', -- url
    '', -- files_url
    400, -- impact_score
    '{"solution": "Easy to use and integrate SDK that does the heavy lifting by enabling developers to add privacy, security and traceability in Cardano apps."}', -- extra
    'Emiliyan Enev', -- proposer name
    '', -- proposer contact
    'https://github.com/ReCheck-io', -- proposer URL
    'Tens of successfully implemented blockchain projects, pilots for the Dutch government, winner in Odyssey hackathon, huge tech expertise.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    273,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=4), -- objective
    'ZooCardano-Stories of Value',  -- title
    'Can we create an NFT for proposals? Catalyst is the greatest event crypto has seen. How do we harness these "Stories of Value''? See icon.',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=4), -- category - VITSS Compat ONLY
    'gHTJRQu6XQtEHUDqjk4cnj1Lb+1LQiTMcvqMcVQHFLI=', -- Public Payment Key
    '300000', -- funds
    'http://ideascale.com/t/UM5UZBf97', -- url
    '', -- files_url
    259, -- impact_score
    '{"brief": "This is part of creating ZooCardano, an interactive and visual microcosm game that can be a source of revenue for the community and a tool to learn more about ourselves and these walled gardens in which we live. How can we onboard value in the beginning of a campaign instead of waiting for voting? What do proposals need in order to to generate greater paqrticipation from the community?", "importance": "Our story is our identity and we, individually and collectively, value these stories. Stories are cultural assets and must be preserved.", "goal": "More users, better user experience, improved UI. Successful implementation of this will be an NFT of NFTs that contain exchanges of value.", "metrics": "Users  \r\nApps  \r\nGame Development  \r\nNFT Suggestions"}', -- extra
    'Q U A S A R', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    274,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'EmancipationStation',  -- title
    'Catalyst lacks the ability to capture the story of our decentralized lives and allow us to organize ourselves to prepare for the unknown fut',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    '/hgflRYwxWoMLDDugWQGT1LXdsoWfDJrx0Vd/P5pcYg=', -- Public Payment Key
    '25000', -- funds
    'http://ideascale.com/t/UM5UZBgAF', -- url
    '', -- files_url
    143, -- impact_score
    '{"solution": "The dapps will be an aggregation of all dapps and software that is brought forth in the past, current, and future proposals."}', -- extra
    'Q U A S A R', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Independent Scholar, caged human, systems migration and idea landscape architect.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    275,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Data sharing for medical AI',  -- title
    'limited amount of experimental data in one company leads to ignorance of rare-events. This is risky for AI performance.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'lBaScisteE6wC72eUFsKmf/EZqUpUxAWm1cw/1pvDEE=', -- Public Payment Key
    '10000', -- funds
    'http://ideascale.com/t/UM5UZBgAb', -- url
    '', -- files_url
    137, -- impact_score
    '{"solution": "Establish a blockchain platform to share and synchronise data from different companies, so that all data can be used for all companies'' AI."}', -- extra
    'yiweizhang1025', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'A teammate works on blockchain. I have medical AI experience.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    276,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=4), -- objective
    'Global Parity Valuation Engine',  -- title
    'Nearly every exchange is based on faulty values making them unfair. Markets need a new baseline and method to deliver fair value for all.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=4), -- category - VITSS Compat ONLY
    'UTCLKdV47ZpBapJvMczLRW230zjfPm/WFZjVWUNfL00=', -- Public Payment Key
    '35000', -- funds
    'http://ideascale.com/t/UM5UZBgBW', -- url
    '', -- files_url
    233, -- impact_score
    '{"solution": "For an engine to record basic human need transaction''s data the world over to create an index, a measure to be used as a baseline of value."}', -- extra
    'Jordan Gitterman', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Vast international experience in direct trade, commodities, natural resources and spawned the first global parity valuation engine.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    277,  -- id
    (SELECT row_id FROM objective WHERE id=5 AND event=4), -- objective
    'Idea to Team Connection Support',  -- title
    'Some folks with good ideas and business acumen cant code; some folks who can code don''t have the business acumen',  -- summary
    (SELECT category FROM objective WHERE id=5 AND event=4), -- category - VITSS Compat ONLY
    '9Fs9K5VdHQdprRCccinpGvzjJc+L8W7+Bg6hnalr+0U=', -- Public Payment Key
    '3000', -- funds
    'http://ideascale.com/t/UM5UZBgEz', -- url
    '', -- files_url
    176, -- impact_score
    '{"solution": "Allow proposal submitters to identify what they need to push their idea or team to the next level."}', -- extra
    'Rfranks08', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '12 years building teams to deliver tech solutions;

an always increasing appreciation that it''s important to know what you don''t know', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    278,  -- id
    (SELECT row_id FROM objective WHERE id=4 AND event=4), -- objective
    'AI Proposal Evaluation and Guidance',  -- title
    'We''re relying on humans to critique an ever growing set of proposals; how can AI provide higher quality feedback and improve results',  -- summary
    (SELECT category FROM objective WHERE id=4 AND event=4), -- category - VITSS Compat ONLY
    'ix19Q78pG5k4eD+gbwA0VF6QIRcCBcVv6Ty0jF0rgvk=', -- Public Payment Key
    '10000', -- funds
    'http://ideascale.com/t/UM5UZBgFN', -- url
    '', -- files_url
    304, -- impact_score
    '{"solution": "1) build data sets about ideas and projects  \r\n2) train AI to critique proposals and give recommendations"}', -- extra
    'Rfranks08', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Me: AI & Data Strategy Consultant @ Big 4 firm; 12 years experience building systems

1x Support: has AI & data strategy experience', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    279,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=4), -- objective
    'No-Code/Low-Code Solutions',  -- title
    'How do we lower the barrier to entry to participating - show what can be built on Cardano without coding',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=4), -- category - VITSS Compat ONLY
    'MmTnCTBbsJfD1oo7POarRHZwLsSmqrOzkQzBlsv6yds=', -- Public Payment Key
    '200000', -- funds
    'http://ideascale.com/t/UM5UZBgFU', -- url
    '', -- files_url
    278, -- impact_score
    '{"brief": "Lets show the world what can be built on Cardano using Cardano''s current low-code/no-code toolsets (e.g., Marlowe Playground). Cardano low-code/no-code tools can be combined with other low-code/no-code development platforms", "importance": "Gartner predicts >65% of software development on low-code/no-code platforms by 2024", "goal": "Portfolio of apps / dapps using Cardano developed via no-code / low-code platforms that can be used to inspire non-technical entrepreneurs", "metrics": "Number of Use Cases Address\r\n\r\nNumber of Solutions Developed\r\n\r\nKnock on Participation\r\n\r\nCost for Value of Low-Code / No-Code Solutions vs Comparable Traditional-Code Solutions"}', -- extra
    'Rfranks08', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)

;

