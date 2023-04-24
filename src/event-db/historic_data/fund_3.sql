--sql
-- Data from Catalyst Fund 3

-- Data Sources see:
-- https://input-output.atlassian.net/wiki/spaces/VIT/pages/2892759058/Catalyst+Fund+Cycle+Releases
-- https://input-output.atlassian.net/wiki/spaces/VIT/pages/2088075265/Fund-3
-- https://input-output.atlassian.net/wiki/spaces/VIT/pages/2155282660/Fund+3+archive
-- https://input-output.atlassian.net/wiki/spaces/VIT/pages/2155282739/F3+General+governance+parameters
-- https://github.com/input-output-hk/catalyst-resources/blob/master/snapshots/snapshots.json

-- Purge all Fund 1 data before re-inserting it.
DELETE FROM event WHERE row_id = 3;

-- Load the raw Block0 Binary from the file.
\set block0path 'historic_data/fund_3/block0.bin'
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

(3, 'Catalyst Fund 3', 'Create, fund and deliver the future of Cardano.',
 '2021-01-06 21:00:00', -- Start Time - Date/Time accurate.
 '2021-04-02 19:00:00', -- End Time   - Date/Time accurate.
 '2021-03-05 19:00:53', -- Registration Snapshot Time - Date/time Accurate. Slot 23404562
 '2021-03-05 19:00:53', -- Snapshot Start - Date/time Accurate. Slot?
 2950000000,            -- Voting Power Threshold -- Accurate
 100,                   -- Max Voting Power PCT - No max% threshold used in this fund.
 NULL,                  -- Insight Sharing Start - None
 '2021-01-13 21:00:00', -- Proposal Submission Start - Date/time accurate.
 '2021-01-20 21:00:00', -- Refine Proposals Start - Date/time accurate.
 '2021-01-27 21:00:00', -- Finalize Proposals Start - Date/time accurate.
 '2021-02-03 21:00:00', -- Proposal Assessment Start - None
 '2021-02-10 21:00:00', -- Assessment QA Start - Date accurate, time unknown.
 '2021-03-05 19:10:00', -- Voting Starts - Date/time not sure because very close to snapshot.
 '2021-03-29 19:00:00', -- Voting Ends - Date/time Accurate.
 '2021-04-02 19:00:00', -- Tallying Ends - Date/time Accurate.
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
    3, -- event id
    'catalyst-community-choice', -- category
    'Community Choice', -- title
    'Which challenges should be launched during Fund5 (March 31st) in order to fulfill Cardano''s mission?', -- description
    'USD_ADA', -- Currency
    500000, -- rewards total
    NULL, -- rewards_total_lovelace
    NULL, -- proposers rewards
    1, -- vote_options
    '{"url": {"objective": "https://cardano.ideascale.com/a/campaign-home/25800"}}' -- extra objective data
)
,

(
    2, -- Objective ID
    3, -- event id
    'catalyst-simple', -- category
    'DApp Creation', -- title
    'What DApps should be built to drive user adoption?', -- description
    'USD_ADA', -- Currency
    250000, -- rewards total
    NULL, -- rewards_total_lovelace
    NULL, -- proposers rewards
    1, -- vote_options
    '{"url": {"objective": "https://cardano.ideascale.com/a/campaign-home/25797"}}' -- extra objective data
)
,

(
    3, -- Objective ID
    3, -- event id
    'catalyst-simple', -- category
    'Developer Ecosystem', -- title
    'How can we encourage developers to build Dapps on top of Cardano?', -- description
    'USD_ADA', -- Currency
    250000, -- rewards total
    NULL, -- rewards_total_lovelace
    NULL, -- proposers rewards
    1, -- vote_options
    '{"url": {"objective": "https://cardano.ideascale.com/a/campaign-home/25805"}}' -- extra objective data
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
    1,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'SoMint - Art 3.0 "A Fresh Approach"',  -- title
    'The current NFT art market is not open to the masses due to lack of education, high gas fees & exclusivity.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'scf0cGdYlC2l7+JD2VmPMeE6R2ybHj65AtL3si5cajg=', -- Public Payment Key
    '20000', -- funds
    'http://ideascale.com/t/UM5UZBemz', -- url
    '', -- files_url
    320, -- impact_score
    '{"solution": "Developing a marketing campaign to educate artists (and collectors) about NFTart."}', -- extra
    'Azeez [Somint]', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Azeez: Marketing professional & creative

Farouk: Designer, curator, artist

James: Educator @Gimbalabs

Rich: Business development @NFT DAO', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    2,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'BC Wildfire Identification',  -- title
    'There are many ways that wildfires start, faster identification and acknowledgement can greatly reduce their impact.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    '/TwpVKojJulnmznvSgM5at0FcVph8P/Cxj8gjbRHFjs=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBd4I', -- url
    '', -- files_url
    238, -- impact_score
    '{"solution": "Utilization of a distributed app to enable incentivised crowdsourcing would greatly reduce the loss of both property and environments."}', -- extra
    'KC', -- proposer name
    '', -- proposer contact
    'https://play.google.com/store/apps/details?id=com.wildfire.wildfire', -- proposer URL
    'I currently own/run the BC wildfire app that allows the general public to both submit and confirm wildfires but it lacks incentivization.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    3,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Cardano ETL: Public BigQuery Data',  -- title
    'Collection and democratization of data in the Cardano ecosystem will be critical for feeding new ideas and measuring critical project KPIs',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    'TQTqoKYVel66CpbzQzJR3Jjzcm2wksWZa9NiaVYrTqo=', -- Public Payment Key
    '30000', -- funds
    'http://ideascale.com/t/UM5UZBd4G', -- url
    '', -- files_url
    337, -- impact_score
    '{"solution": "Cardano ETL will support transformation of blockchain data into convenient formats like JSON Newline, GCP PubSub, and relational databases."}', -- extra
    'Bourke Floyd', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I''ve been working in the mobile gaming space for 8 years creating client and server architectures at petabyte scale with both GCP and AWS', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    4,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Orphic CLI Connector to BigCommerce',  -- title
    'Developers need a way to easily connect their projects or work to a cryptocurrency and our CLI connects them to Ruby, Golang, and NodeJS.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    'aZNA3CRYv76+1A5mmMYXB320/BWjZOwTCVQ86nI5bS0=', -- Public Payment Key
    '20000', -- funds
    'http://ideascale.com/t/UM5UZBd4F', -- url
    '', -- files_url
    162, -- impact_score
    '{"solution": "Our CLI connects various APIs like BigCommerce Checkout-JS and web utilities like Ruby on Rails. These CLIs are built in Ruby, Golang, JS."}', -- extra
    'Kyle OBrien', -- proposer name
    '', -- proposer contact
    'https://orphic.space', -- proposer URL
    'Our founder has extensive experience with GraphQL, REST APIs, and language-related projects. We also have a team of volunteer developers.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    5,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Cardvrytin Mobile App to Drive Digi',  -- title
    'The E-commerce industry is growing by the day despite the pandemic. We need to come up with an app to solve financial transactions issues.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'hXBKVf/sTnMuNWAc2aOd0+qFU79B7ftkMd3il8byC00=', -- Public Payment Key
    '150000', -- funds
    'http://ideascale.com/t/UM5UZBd4C', -- url
    '', -- files_url
    182, -- impact_score
    '{"solution": "In my Blocking Mobile Technologies academy, I preach much about Cardano and all that students ask is how they can buy cardano with a FIAT."}', -- extra
    'stanley okwu', -- proposer name
    '', -- proposer contact
    'https://blockchainmobiletech.com/', -- proposer URL
    'We have experience in the mobile money business and also an instructor in the blockchain arena. BMT wants to focus on blockchain industry.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    6,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=3), -- objective
    'Atala PRISM DID Mass-Scale Adoption',  -- title
    'What will drive mass-scale adoption of decentralized IDs on Cardano?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=3), -- category - VITSS Compat ONLY
    'rB4BKgGCr1RKb/wDMnQ61Hxzaky72ILciusfAMFtOws=', -- Public Payment Key
    '250000', -- funds
    'http://ideascale.com/t/UM5UZBd4A', -- url
    '', -- files_url
    500, -- impact_score
    '{"brief": "Since this is about a challenge and not a proposal, I think some collaboration and idea exchange on how best to frame this is an important consideration in the process with community engagement. My first pass is something like this:\n\n\n\nCHALLENGE:\nPropose a means or way to drive mass-scale adoption of Atala PRISM DID as an easy-to-use onramp to the Cardano ecosystem. This can include any method that will offer users or entities a decentralized ID to start defining their digital self-sovereignty. Think about how this is implemented at scale drawing millions of users, who, once onboard, can become customers for dApps, tools, DeFi and other services.\n\n\n\nNOTE: Please help refine this, it works better when we all have input! If anyone is truly passionate about this and wants to be a co-submitter, let me know. - Rich", "importance": "Atala PRISM DID is a gateway to the Cardano blockchain and ecosystem. Building technology and marketecture that on-boards users is crucial.", "goal": "Many high quality ideas will be proposed that can substantially grow Cardano''s user-base, network-utility, network-value & ADA circulation.", "metrics": "\\* The number of proposals that directly address mass-scale adoption in a quantifiable manner.\n\n\\* The quality of proposals measured by community chosen metrics.\n\n\\* Results of any Fund5 funded projects significantly impacting mass-scale adoption of DID''s and their utility on the Cardano blockchain."}', -- extra
    'rich', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    7,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Comprehensive NFT Framework Collab',  -- title
    '42 Catalyst proposals address NFT point solutions—yet **we lack an overarching DApp strategy** to dominate as the de facto NFT platform choice.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    '+UMAlGU/02ySyqMPDpckDckD4S9iVX8ziqQC2uKvSuM=', -- Public Payment Key
    '48965', -- funds
    'http://ideascale.com/t/UM5UZBd37', -- url
    '', -- files_url
    422, -- impact_score
    '{"solution": "Coordinate an **NFT DAO that implements all NFT DApp requirements** to build a penultimate composable NFT Framework\u2014**a WAX Killer on Cardano.**"}', -- extra
    'rich', -- proposer name
    '', -- proposer contact
    'https://github.com/kopcho/NFTDAO', -- proposer URL
    'The 42 **proposing teams cover the range of skills and experience needed** to build the best possible solution to dominate the NFT space.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    8,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Customizable DeFi Data Streams',  -- title
    'Developers will not build on Cardano if they do not have robust and affordable API endpoints from the blockchain. How can we solve this?',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    'SepqmwoVD2FmpWb3hA0Bp3VUkoyC44hqiwkrp8CwBTU=', -- Public Payment Key
    '75000', -- funds
    'http://ideascale.com/t/UM5UZBd3z', -- url
    '', -- files_url
    196, -- impact_score
    '{"solution": "Construct a comprehensive data system giving Cardano developers access to blockchain data similar to Etherscan for the Ethereum network."}', -- extra
    'Alexander', -- proposer name
    '', -- proposer contact
    'https://github.com/olealgoritme', -- proposer URL
    'We are a team of software architects with 50+ years of combined experience in data infrastructure and back end development.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    9,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=3), -- objective
    'Decentralization of Public Services',  -- title
    'Most public services are sub-contracted at 3-5x cost. How is community self-governance best achieved via delivery on a utility platform?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=3), -- category - VITSS Compat ONLY
    'sRjQJLX2wMl7ERMKaYaKDRSLRdelEL6zMHviz/zT9fs=', -- Public Payment Key
    '300000', -- funds
    'http://ideascale.com/t/UM5UZBd3w', -- url
    '', -- files_url
    500, -- impact_score
    '{"brief": "It is indeed a challenge to any community to redefine the very concept of itself in the apparent absence of applicable, historical precedent. Without a relatable narrative, even the most imaginative minds can be forgiven for falling back on legacy concepts and inherited structures, all while seeking to break the mold and redefine it. And according to what model?\n\nWhat does it even look like, to live in a truly decentralized, self-governing community, when all we''ve ever known is an essentially top-down, command-and-control infrastructure?\n\nSure, there''s 32 flavors to choose from at Baskin-Robbins, but one''s choices narrow significantly from there, the more important they become. A truly trustless technology carries the promise of liberating humanity, however, and so this implies the ++unraveling++ of existing modalities of governance, far more so than the creation of new ones.\n\nIf the goal is the strengthening of self-governance, and the shaping of legislation and commercial standards accordingly, I posit the best approach to be a practical one, in keeping with the true bottom-up nature of decentralization. As for historical precedent, who would''ve guessed the answer to that age old question, *\"But without government, who will build the roads?\"*, can be found in a piece of near-forgotten history from over 180 years ago?\n\nAlexis de Tocqueville was an aristocrat and political scientist from the Courts of Versailles, France, who eventually became Minister of Foreign Affairs, and he is universally acknowledged as the father of sociology and social anthropology throughout most all of western academia. As with many others in Europe at the time, de Tocqueville was mesmerized with this strange new nation called America across the sea.\n\nIn 1831 he finally visited and chronicled his travels within his famous, ''Democracy in America'', published in 1835; the second volume published another 5 years thereafter. In it, de Tocqueville described the astonishment with which he found a near total absence of government in nearly every township he paid visit to, save for the local post office. This was unfathomable to him, for in his native France there were government offices everywhere, with breadlines feeding the poor and destitute, stretching for blocks\u2026 By contrast, in America, what little he could find of the poor and needy were all dutifully cared for by an abundance of charitable organizations; the people keeping 100% of their earnings while looking after their own communities themselves. The discrepancy in literacy rates also astounded him, with barely 10% who could read in his native France, compared with the over 90% literacy rate he found in the States.\n\nPeople maintain that charities could never sustain the needs of the poor today, but if you try to feed the homeless in most cities you will be arrested!\n\nA better way of phrasing the aforementioned question of *\"Who will build the roads?\"* isn''t by hearkening to any notion of zero government, so much as highlighting its replacement with self-government instead. Roads and public services were of course funded directly by community-minded citizens in the old Town Hall meetings of New Hampshire, for example, without any of the trustless automation that can now be leveraged. Centralized, top-down administration of these services have become so intertwined with our very concepts of how communities must operate, it''s not surprising it was a source of bewilderment to Monsieur de Tocqueville just as it is now for us today, and so many years later\u2026\n\nWhat''s perhaps more surprising is why you''ve probably never even heard of de Tocqueville until now. Given these historically remarkable findings from early American society, and his being recognized as creating the science of anthropology itself. Yet even if one studied the subject now, it would probably surprise one even more to discover his near-complete absence from the books currently in print\u2026\n\nAs former Secretary of Education under Reagan, Gary Bauer, openly declared: for every $1 collected in tax for the purpose of education, only $0.25 ever goes to the actual schools to pay for the teacher''s salaries and books. A full 75% of it simply lines the walls of the ever expanding bureaucracy there in D.C., like some malignant tumor! I was so stunned by this announcement when I finally heard it (many years later), I personally contacted his offices to confirm it, and he told me now it wasn''t even $0.20 out of every dollar\u2026\n\nI then researched the average expenditure in taxes per pupil in D.C. for that year, and it was over $26,000 per head - more than a year''s tuition at Montessori! But private schools are just as dependent upon government licenses to teach as those that are declared public, and the books, now bereft of any hint as to how communities can indeed thrive much better under the liberty of self-governance, reflect the purposeful neglect a top-down mode of governance imposes for its own sake just the same. But as far as public services are concerned, I know education is still perhaps a bit too delicate a subject for most to accept the decentralization of. Best keep it to street cleaning and such for starters!\n\nThe concept of a decentralized, self-governing community is so foreign, most people assume there''s no historical context by which it may be easily conceptualized. The fact de Tocqueville is acknowledged as the father of socio-anthropology, and yet had his own observations removed from all the books on the subject, says something as to why\u2026 but what he recorded is proof this has been achieved before, and was so powerfully successful compared to the top-down models of governance, its example had to be censored.\n\n*\"A Republic, if you can keep it.\"* ~ Benjamin Franklin\n\nCardano very much represents an opportunity for, not just a return to sound money, but a return to a truly decentralized community of self-governance, such as the Republic to which Franklin referred.\n\nAlso Ben: *\"When the people find that they can vote themselves money that will herald the end of the Republic.\"*  \n\nThis is unfortunately beginning to happen now, and it''s easy to understand why. When the ability to earn oneself a living is removed, of course people will vote themselves money ad infinitum, but this must be acknowledged as an extremely urgent and dangerous situation! Especially when the concept of decentralization, while generally positive, still resides somewhere in the \"nice to have\" category of perspective; much like the other benefits of blockchain, such as immutability, security, inclusiveness, etc. People are still looking at decentralization and thinking, yes that''d be \"nice to have\"\u2026\n\nWell let''s take a look at what we *do* have at the moment:\n\n\\- Central Credit Monopoly  \n\\- Media Mind Control  \n\\- Destruction of Private Industry\n\nEssentially, Planks 5, 6, & 7\u2026\n\nDecentralization *is not* a \"nice to have\", it is a **must**.\n\nAs mentioned in the metrics section, this challenge is for proposers from all nations, and a truly decentralized, self-governing society is the goal. America is where we may find the greatest, socio-anthropologically confirmed, recorded example of decentralization''s outstanding success. By contrast, another example of what''s being hailed as a \"self-governing\" success, are the new, private company governed townships being offered in Nevada, where technocratic Lord-Barons pose to make a dubious return (see https://reason.com/2021/02/08/tech-companies-could-form-their-own-governments-under-a-new-nevada-proposal/).  \n\nPower always seeks to maintain its perch, and expand it where ever it may. The technology available to us now will be wrestled over to this end, and this may be our last chance to show the world how much better a truly self-governing society can and would be, should that power be finally pushed to the edges.\n\nThank you for your consideration and your time!\n\n*\"A man''s admiration for absolute government is proportionate to the contempt he feels for those around him.\"* ~ *Alexis de Tocqueville*\n\nhttps://sleepingnatives.org/projects.html#alexis", "importance": "*Community mindedness* today risks veering from its original meaning. Self-governance is, by definition, a bottom-up, not top-down, approach.", "goal": "Elected authorities from amenable jurisdictions become local heroes for greatly reduced taxes. Private companies are paid more and on time.", "metrics": "\\* Please note: this challenge is open to proposers from all continents and nations.\n\nProposers are welcome to explore and weigh the merits of transaction metadata, smart contracts, tokens, smart markets, and/or any combination thereof in the determination of how best to align services with specific zones & localities (such as street cleaners and/or garbage collectors, for example), contracted for a given term.\n\nCollaboration will prove a key factor here. A project of this scope will require talent and expertise across a wide array of skillsets and disciplines. Prospective proposers are encouraged to engage (across the project''s Discord channel or elsewhere) in order to build a team that can arguably, if successful, help change the tide of history (more on that below)!\n\nOf greatest initial importance will be the opt-in, by at least one local government authority and a service provider, to accommodate a Proof Of Concept within a pilot jurisdiction. The boon from a successful POC, for any politician (as a now-lauded and pioneering visionary), will yield them a near certain re-election and serve as a model for blockchain efficiency, with subsequent demands for its expansion and implementation elsewhere.  \nKey metrics:\n\n*   The proposed solution must demonstrate a substantial savings to be had by the taxpayer, as compared to the amount local governments set aside for the contracted service from a given tax (like property taxes). Each local government participant is expected to act as an honest broker in these arrangements, redacting 100% of the respective service(s) being off-loaded; the key word being \"expected\". This comparison should be made between the projected operating costs of the proposed solution, once deployed, versus the status quo. Initial development costs will no doubt skew this comparison in the beginning, so it''s important to keep in mind the savings from an ongoing perspective, especially when instantiated across multiple jurisdictions.\n*   Initially, service providers themselves should expect remuneration to not only be significantly greater than when contracted by their local authorities, but also to be paid far sooner than is typically the case for any government contractor.\n*   As time progresses, new businesses wishing to compete within the service provider''s space should also be accommodated for proposals to the local community from within the deployed solution, so that citizens may collectively choose for themselves which provider the service will be granted, and for how long (with a likely quorum being required for the contracting of a given area or district); much like a digital \"Town Hall\" for each local community, for citizens to decide on local matters.\n*   As with any advancement in automation and technology, government jobs can also be expected to be made redundant/nonessential. While this isn''t necessarily the kind of key metric they themselves might look forward to, private markets have born the brunt of such consequences of innovation to a disproportionately larger degree, and for far too long. If there''s any sector in need of economizing and reducing waste, for sure it''s within the government. What''s good for the goose is surely good for the gander!\n*   Last but not least, while politicians themselves may regret the latter, they can most assuredly expect a much happier electorate as a result, redounding to a re-election. It really depends if they are for big government, or small."}', -- extra
    'Philip de Souza', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    10,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=3), -- objective
    'a better catalyst',  -- title
    'how can we design the best catalyst experience?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=3), -- category - VITSS Compat ONLY
    '5CJ9c3tSUM4OjuKmFxoAul8IVDM6NJGky3sBZ9mhYVI=', -- Public Payment Key
    '100000', -- funds
    'http://ideascale.com/t/UM5UZBd3v', -- url
    '', -- files_url
    500, -- impact_score
    '{"brief": "how can we design the best catalyst experience?", "importance": "better feedback loops and trust building. if communication, collaboration and compensation is solved, we can finally operate \ud83d\udcafdecentralised", "goal": "competing diverse trusted interface(s) for ideation, iteration and interaction - with common member iDentities + treasury smart contract.", "metrics": "**Communication** - *quality of interactions (comments/questions:edits/iteration) from early stage to voting stage*\n\n**Collaboration** - *quality of connections (frequency of collaborator communication / proposal* *iterations)*\n\n**Compensation** - *quality of (monetary) exchange.*\n\n*micropayments to commenters(comments with proposer responses viewed as valid contributions, a multiplier would be if an iteration was credited to their comment/question/feedback.*\n\n*as well as micropayments to collaborators (for forming collaborations and creating proposal iterations).*\n\n***ultimately** having the treasury smart contract that disperses the proposals funding request integrated into this final stage would be the ideal. this would then allow for the loop back to happen once proposal comes back for milestone-based later stage funding after completion of its proof of concept - or for it to continue on with it''s already created community if it has already achieved financial sustainability.*"}', -- extra
    'A', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    11,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Nmadi Space: A Digital Universe',  -- title
    'Creative, sandbox-style games don''t share a common physics system with shared resources subject to conservation laws of matter and energy.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'E6LPTtyOu9mfGCPmu1ncw+ZIUkXQTBlZrbNvJJsC2aY=', -- Public Payment Key
    '19000', -- funds
    'http://ideascale.com/t/UM5UZBd3u', -- url
    '', -- files_url
    218, -- impact_score
    '{"solution": "An open source distributed physics system that allows in-game development of sub-games, inventory on Cardano, and conservation of resources."}', -- extra
    'Ken Stanton', -- proposer name
    '', -- proposer contact
    'https://github.com/thistent', -- proposer URL
    '15 years studying distributed system architecture  
20+ years studying physics  
30 years imagining new worlds  
Pony / Erlang / Elixir / Rust', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    12,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Food traceability solution - Africa',  -- title
    'African food supply chain is made of smallholder farmers who often do not get the due reward for their produce thereby leaving them poorer.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'NCKWeGjozVpEQLfwYM1A46LehLHJ/bFXepy+lOjmMYE=', -- Public Payment Key
    '3500', -- funds
    'http://ideascale.com/t/UM5UZBd3t', -- url
    '', -- files_url
    309, -- impact_score
    '{"solution": "Mapping potential opportunities in Nigeria to apply a pilot using the outcomes of \"EMURGO Traceability Solution\" implemented in Indonesia."}', -- extra
    'gwendal.ledivechen', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'ABCD team (proj. man/engineers) + one developer who graduated in Agricultural Extension and Rural Development at Obafemi Awolowo University', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    13,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'play_x 😉😵😌 - Adult Role Play 🎭 ',  -- title
    'the loneliness epidemic leads to a higher risk of premature death.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'Mp7n2O9Esb+Ask5BsjfgvNmLfWFzQOqZ6mH5izzhms0=', -- Public Payment Key
    '1000', -- funds
    'http://ideascale.com/t/UM5UZBd3n', -- url
    '', -- files_url
    171, -- impact_score
    '{"solution": "a decentralised app that enables fun adult role playing with anonymity and accountability."}', -- extra
    'A', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'i am human. so i''ve got that going for me. i also have decent related skills in designing user experiences.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    14,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'GreenSavings (PoupVerde)',  -- title
    'Every day, liters of cooking oil waste are incorrectly disposed of by the population.They pose serious risk of pollution to the environment.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'qsuvphIXxqT4hzu/0wimZX2cNS/5X8F5YZmdZJTmMY0=', -- Public Payment Key
    '21163', -- funds
    'http://ideascale.com/t/UM5UZBd3j', -- url
    '', -- files_url
    357, -- impact_score
    '{"solution": "Development of a dApp for the collection of waste used cooking oil, in exchange ADA, which will be used for the production of biodiesel."}', -- extra
    'Ático Mismana', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'The team has transversal skills in the areas of socio-environmental entrepreneurship, education, blockchain and software development.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    15,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Cardano Algorithmic Credit Scoring',  -- title
    'ADA holders with extensive, reliable crypto lending history want access to more competitive loan products and working capital.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'LADCQSpBwh1HnuxQdgoth0kgLBrlNhSqIANLt59BSmA=', -- Public Payment Key
    '20000', -- funds
    'http://ideascale.com/t/UM5UZBd3c', -- url
    '', -- files_url
    340, -- impact_score
    '{"solution": "Credmark is building a borrowers portal to facilitate account attestation, aggregating lending history across multiple blockchains."}', -- extra
    'Neil Zumwalde', -- proposer name
    '', -- proposer contact
    'https://credmark.com', -- proposer URL
    'Neil: 9 years experience in Fullstack dev, Paul: founded 5 startups, 3 with exits. Advisors from leadership at Moody''s, FICO, and Coinbase.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    16,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Scalable fake news detection DApp',  -- title
    'The uncontrollable propagation of fake news through digital media poses direct harm for society in the long term and individual cases now.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'x2GH1QHY4yVgCE1E1BXd1zSiNz/gb8DCHn4AsUkcjXo=', -- Public Payment Key
    '46400', -- funds
    'http://ideascale.com/t/UM5UZBd3X', -- url
    '', -- files_url
    307, -- impact_score
    '{"solution": "Leveraging Cardano, dynamically construct workflows from third party AI algorithms and data sources for real-time evaluation of content."}', -- extra
    'kabir', -- proposer name
    '', -- proposer contact
    'https://nunet.gitlab.io/fake-news-detection/catalyst-fund3-proposal/', -- proposer URL
    'Kabir: PhD in AGI and CS; 15y+ project management

iCog-Labs.com: AI & software R&D from 2012

SingularityNET.io: AI on blockchain from 2017', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    17,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'TrashApp',  -- title
    'Litter. It''s everywhere and it''s awful for the earth/wildlife. Few people volunteer to clean it, but what if anyone could get paid to do it?',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'uctIIYvch8rokl3Nb6bNpyHKzuNPZJFbfAvQe1KDWn0=', -- Public Payment Key
    '30000', -- funds
    'http://ideascale.com/t/UM5UZBd3S', -- url
    '', -- files_url
    289, -- impact_score
    '{"solution": "A simple DApp that enables peer to peer litter collection using photos, timestamps, and a bounty payment system."}', -- extra
    'Sam', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Owned and operated several small businesses in real estate and e-commerce. Leveraging partners/collaborators experience in DApp development.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    18,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Impact the Art Industry with ArtAcy',  -- title
    'Many artists have faced problems with unauthorized copies of their works, and it hurts emerging artists as well as established artists.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'LuRwwfFonNKcbPIUkTRPD5+oaWi149GCCCdij73wQ/s=', -- Public Payment Key
    '35000', -- funds
    'http://ideascale.com/t/UM5UZBd3R', -- url
    '', -- files_url
    348, -- impact_score
    '{"solution": "Platform that will engage and educate artists and investors, ready to provide fast-tracked liquidity and confidence in the art industry."}', -- extra
    'Izis Filipaldi', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Izis Filipaldi: Computer Engineer +10y / Agile Coach +5y / Art investor +6y  
Pedro Tramontin: Computer Engineer +10y', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    19,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Marlowe and Plutus Mobile',  -- title
    'Many people have access to cell phones, but not computers which makes programming more difficult to learn in developing nations.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    '7ym/aKoXjz+CAgyQu4icDzvtCA/oeXgURD1b102rrug=', -- Public Payment Key
    '19000', -- funds
    'http://ideascale.com/t/UM5UZBd3O', -- url
    '', -- files_url
    267, -- impact_score
    '{"solution": "To build a mobile interface for making **AST**s that focuses mainly on Cardano languages. Also a server to manage compilation and collaboration."}', -- extra
    'Ken Stanton', -- proposer name
    '', -- proposer contact
    'https://github.com/thistent', -- proposer URL
    '19 years Linux experience

Programming language design and developer experience

Erlang / Elixir / Elm / Rust / Pony / Nix

Linguistics', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    20,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Better Ideascale But Open-Source ',  -- title
    'Ideascale is not pleasant to use, it sucks. As users, we feel more often restricted than empowered on this platform when proposing new ideas',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    'toK7qbfwnFa/mIyqU7aIR2cW/TnS1X8AYJlDL7X2oJI=', -- Public Payment Key
    '55000', -- funds
    'http://ideascale.com/t/UM5UZBd3A', -- url
    '', -- files_url
    224, -- impact_score
    '{"solution": "A native idea generation platform dedicated to Cardano blockchain. Open-source, simplified, gamified, modern, mobile, made to scale global."}', -- extra
    'Carolin Taling', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'More than 20 years of experience in software engineering, design and marketing. Check under to read more about the team and our skills.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    21,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'ReCheck.Me',  -- title
    'The adoption of Cardano blockchain is impeded without tools for digital sovereignty, identity management, secure data sharing & el. signing.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'uvk371+zVKd4rv6Um9OPiYAp/pcTRyRH/1zXX0nmQdE=', -- Public Payment Key
    '40000', -- funds
    'http://ideascale.com/t/UM5UZBd2z', -- url
    '', -- files_url
    252, -- impact_score
    '{"solution": "DApp to manage your personal data & verifiable credentials in a way that you are in centre, own your data and control access to information."}', -- extra
    'Emiliyan Enev', -- proposer name
    '', -- proposer contact
    'https://github.com/ReCheck-io/recheck-sdk', -- proposer URL
    'Four and a half years in the development of blockchain solutions, multiple awards, tens of implemented projects, pilots for Dutch Ministry.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    22,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Authentication for DeepFake Defense',  -- title
    'Deepfake videos are a type of AI generated video that creates a fake video of a person that is convincingly real. This is dangerous.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'PHmj5HbZYqxPlBz2sFRkI3rlpr/StM2upCKhzn4iYeY=', -- Public Payment Key
    '12000', -- funds
    'http://ideascale.com/t/UM5UZBd2t', -- url
    '', -- files_url
    206, -- impact_score
    '{"solution": "We will create a cryptographic proof on Cardano that verifies videos are real by connecting their blockchain ID."}', -- extra
    'the_fig_monster2', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Cryptography student, website development, blockchain technologist.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    23,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'DARP: Cardano''s Address Name Servce',  -- title
    'Complex addresses (e.g. cryptocurrency, network, physical) are difficult to share and remember, which adds risk to their use.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    'F5I7HUhk3FPiT0bL9gc/hhcC0wv+KGqyfQ9rhF27BlU=', -- Public Payment Key
    '57500', -- funds
    'http://ideascale.com/t/UM5UZBd2o', -- url
    '', -- files_url
    190, -- impact_score
    '{"solution": "The Decentralised Address Resolution Protocol (DARP) uses easy to remember address names to help mitigate risks & improve user experience."}', -- extra
    'Phil Lewis', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Please review my experience on LinkedIn at https://www.linkedin.com/in/phillewisit/', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    24,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Goguen Launch Global Hackathon',  -- title
    'How do we raise awareness and excitement about the Goguen launch and accelerate dApp development on Cardano?',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    'XjK8p6fplZ/PRTjYmICLODAp1r+RWOcjg/vZdRkVQ/4=', -- Public Payment Key
    '80000', -- funds
    'http://ideascale.com/t/UM5UZBd2l', -- url
    '', -- files_url
    248, -- impact_score
    '{"solution": "Work with Hackethon.com one of the biggest providers of turnkey Hackathon services to produce a world class Hackathon event."}', -- extra
    'gshearing', -- proposer name
    '', -- proposer contact
    'https://docs.google.com/presentation/d/1shUEsyHLD-0kAO9ZtjDNpy82D-6eqD3WflbLarEcYU8/edit#slide=id.g367d6047d4_0_3101', -- proposer URL
    'Marketing professional, Community manager, Entrepreneur', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    25,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Infura-like Cardano API IaaS',  -- title
    'Running and developing custom backend infrastructure is expensive, time-consuming and slows down development of crucial parts of software.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    's/cB2CiYMhnMJFi179F04S0C1cPLCv8vCJ8kZz4HAc0=', -- Public Payment Key
    '60000', -- funds
    'http://ideascale.com/t/UM5UZBd2f', -- url
    '', -- files_url
    306, -- impact_score
    '{"solution": "Provide fast, rich and reliable API access to Cardano for developers and applications through HTTPS and websockets on mainnet and testnets."}', -- extra
    'michal.petro', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Our open-source Adalite backend is used by numerous wallets and developers. Vacuumlabs has 200 engineers and experienced Cardano team.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    26,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Complete Cardano Training Center',  -- title
    'Cardano knowledge is spread everywhere. People don''t know where to start & how to learn. There is no simple way to prove/assess your skills.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    '1qb174RX405oxC5pj+xFAQl5Q37MPqnnzFhuyWEFRNA=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBd2X', -- url
    '', -- files_url
    252, -- impact_score
    '{"solution": "Native Cardano e-learning platform allowing IOG, CF, etc. to create interactive courses, quizzes and credentials on the blockchain."}', -- extra
    'Carolin Taling', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'More than 20 years of experience in software engineering, design and marketing. Check under to read more about the team and our skills.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    27,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Cardano on the Rocks',  -- title
    'Blockchain seems very virtual and a general term to newcomers. Special features are unclear, and mostly only described as laudatory claims.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    'fzKDdI5d/xhF9ualRUQPZuOhiSO6o0E17FiTpKYgRYc=', -- Public Payment Key
    '3900', -- funds
    'http://ideascale.com/t/UM5UZBd2U', -- url
    '', -- files_url
    239, -- impact_score
    '{"solution": "With energy-efficient, inexpensive small computers, superior technology should be presented in a very visual and marketing-supporting way."}', -- extra
    'gufmar', -- proposer name
    '', -- proposer contact
    'https://github.com/clio-one/cardano-on-the-rocks', -- proposer URL
    '20 years of IT services', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    28,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Ada Tx to Trigger IoT + IO HW Spins',  -- title
    'No solution exists to enable arbitrary IoT IO actions as a result of receiving payment in Ada (vending machines, gated access, automation)',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'Iy2N6W7I8EkoMkW0JJXCczLpgFtacEgXDhQwv9mk1RE=', -- Public Payment Key
    '33000', -- funds
    'http://ideascale.com/t/UM5UZBd2P', -- url
    '', -- files_url
    477, -- impact_score
    '{"solution": "Adosia will build new open HW and integrate monitoring of Ada payments into the Adosia IoT platform to enable custom IoT device triggering"}', -- extra
    'Kyle Solomon [FROG]', -- proposer name
    '', -- proposer contact
    'https://adosia.com', -- proposer URL
    'BSE EE, 5 yrs semiconductor sales, built/pivoted ad-tech startup into open hw IoT init, ICO vet, SPO

https://www.linkedin.com/in/kesolomon/', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    29,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Wadspare',  -- title
    'Africa has lots of creative minds who want to establish the "next big thing" but lacks access to capital to fund these ideas.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'GPyvNoRSxc3znH3Q+zw6IIiE3k34Hcsi0OM231ekHss=', -- Public Payment Key
    '20000', -- funds
    'http://ideascale.com/t/UM5UZBd2I', -- url
    '', -- files_url
    132, -- impact_score
    '{"solution": "Wadspare is creating a fundraising platform built on the blockchain for startups."}', -- extra
    'brown gift', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Wadapare Founders have been in the financial institution for a long time.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    30,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Secured Ticketing System For Events',  -- title
    'Most of the ticketing systems are heavily centralized and illiquid. Also are in high risk of duplication, fraud and hassle.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'VVJBvTo6774zTvIxXK5dHDCsvJ3PyxX7oXDf24As88A=', -- Public Payment Key
    '55000', -- funds
    'http://ideascale.com/t/UM5UZBd2F', -- url
    '', -- files_url
    330, -- impact_score
    '{"solution": "Complete ticketing service, allowing users to create, sell and secure tickets simply from any device. At low cost. Using Cardano blockchain."}', -- extra
    'Carolin Taling', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'More than 20 years of experience in software engineering, design and marketing. Check under to read more about all the team and our skills.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    31,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=3), -- objective
    'A for ADA Cryptoalphabet 4 children',  -- title
    'How to increase general awareness about Cardano and cryptocurrencies?

How to make fun community-building incentives?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=3), -- category - VITSS Compat ONLY
    'lTG+8+2O/bmLz7uPYm4FxZgALctZTzRFvEGGNqpO8Qg=', -- Public Payment Key
    '4800', -- funds
    'http://ideascale.com/t/UM5UZBd1p', -- url
    '', -- files_url
    500, -- impact_score
    '{"brief": "A for ADA\n\nB for Bitcoin\n\nThe community-driven effort of creating the crypto-alphabet for the children. The community will vote and express an opinion, volunteer to create rhymes for particular letters and drawings.\n\nWe all agree is that A is for Ada, B for Bitcoin, But D might be for Dash or Decred?\n\nH might be for Horizen or Holochain\n\nT for Tezos or Telos\n\nWhat letter to choose? Let the community decide.\n\nEither by voting or \"supporting\" a particular letter, the community can decide. I.e. if some of you want Horizen to \"win\" the letter H - then you can send any amount of ADA to the project''s address. Then if suddenly Hedera Hashgraph fan appears and start competing with you - the competition to the letter might continue for hundreds or thousands of ADA.\n\nThis crypto-alphabet will be published as Book and distributed for all of the community - first of all, those who take an active part in Project Catalyst. Then as presents to different Cardano meetup attendees, future conferences, and summits, but also it will be sold both as digital and as a paperback. As this project is applying for treasure funds, so the potential profits will be used to rewards active participants of the book creation, additional creation of book supplies, and maybe reserve fund for future activities. I know a publisher who agreed to do this project in my country, and have also their distribution network - so also the book will be sold for people outside of crypto area, which is good as it brings additional brand awareness and potential adoption.\n\n  \n\nThere are 2 option for illustrations:\n\n1\\. Children will draw the illustrations, first of all children of people from Cardano ecosystem, but it can go further to other communities\n\n2\\. We''ll find a designer (it''s not a problem, when the text is ready, each letter - represents each crypto, so it won''t be hard for designer to draw illustration based on it). Maybe the designer from community will be willing to join this project.\n\n  \n\nI''m putting the budget of $4800 USD which will be enough to print 2000 copies of book (well, maybe we should print more) and rewarding all the people who contributed (designers, text creator) and book shipping costs\n\nMany aspects of the process are up for the community to decide", "importance": "The earlier to start financial education - the better it will be in the long-term.\n\nCryptoalphabet - community-created Alphabet for children", "goal": "As community defines it. I.e. If the general awareness will increase, and there will be high demand for \"Cryptoalphabet\"", "metrics": "\\- Number of people engaged into the creation of Cryptoalphabet\n\n\\- Number of people interested in getting this book\n\n\\- Number of book sales outside of Cardano community\n\n\\- Number of books sales outside the crypto space\n\n\\- Number of popular media who covered this story\n\n\\- The revenue generated by the book sales"}', -- extra
    'Andrii Voloshyn', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    32,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'ADA in 4000 Crypto ATMs Globally!',  -- title
    'You cannot currently buy ADA from any Bitcoin ATMs. We will change that if our proposal is chosen',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    'DtL3OZK/JGObRJy1jOs4ChRizX+nlbbt9lgv7Zqsvow=', -- Public Payment Key
    '7500', -- funds
    'http://ideascale.com/t/UM5UZBd04', -- url
    '', -- files_url
    270, -- impact_score
    '{"solution": "We will integrate ADA into the General Bytes Bitcoin ATMs by developing an extension for ADA in the open software software."}', -- extra
    'bryan', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I have been operating ATMs since 2018 and I run the Cardano Abu \[CABU\] stake pool. I''ve also been involved in Cardano for the last 3.5 years', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    33,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=3), -- objective
    'Project Catalyst User Experience',  -- title
    'How can we improve the experience of Project Catalyst users in 6 months or less?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=3), -- category - VITSS Compat ONLY
    'JX4kKfDjYGRGT11G4rkjsp96h+owujCjHIBqB3b3R0E=', -- Public Payment Key
    '100000', -- funds
    'http://ideascale.com/t/UM5UZBd0a', -- url
    '', -- files_url
    500, -- impact_score
    '{"brief": "Project Catalyst''s ability to improve and evolve will have a huge impact on our decentralized autonomous organization''s return on intent; this challenge incentivizes our community to deliver some of those improvements. This campaign aims to test if a $100,000 Fund 5 challenge can incentivize community members to advance the Project Catalyst user experience in 6 months or less from when proposers receiving funding. **User experience is critical to retain community members over time and increase user impact within an organization (see links below for a few supporting references that highlight how user experience is an important in this context).** If the challenge proves successful, it could be renewed each fund and perpetuate measurable improvements to user experience for our organization into the foreseeable future. Lets improve the user experience as a community and measure it.  \n**Third party resources that highlight how important user experience is to organizations:**\n\n*   https://www.mckinsey.com/business-functions/mckinsey-design/our-insights/the-business-value-of-design\n*   Highlight from McKinsey: \"Design flourishes best in environments that encourage learning, testing, and iterating with users\u2014practices that boost the odds of creating breakthrough products and services while simultaneously reducing the risk of big, costly misses.\"\n*   https://knowledge.wharton.upenn.edu/article/user-experience-reimagining-productivity-and-business-value/\n*   Highlight from Wharton link: \"If Amazon or Google sets the standard for ease of use, that is what \\[people\\] expect everywhere \u2013 in all categories.\"\n\n**A third party reference to offer suggestions for good practices in crypto user experience:**  \n\n*   https://www.cryptouxhandbook.com/\n*   Highlight from the handbook: \"The goal is always to be highly considerate of the situation the user is in and what their goals are, and then help them be successful. Talking to users and diving deep into analytics to understand behavior are invaluable.\"\n*   Created by Christoph Ono, who according to his website has worked with Monero. See his website referencing his previous work here: https://www.germanysbestkeptsecret.com/\n\n**References that showcase how Square Crypto has been prioritizing user experience in the crypto space:**  \n\n*   https://www.coindesk.com/square-crypto-grant-development-bitcoin-design-ux\n*   Scroll to \"Designer Grants\" https://squarecrypto.org/#grants\n\n\\*I have no connections to any of the references listed above\\*", "importance": "Project Catalyst''s ability to improve and evolve users'' experiences will have a huge impact on our organization''s return on intent.", "goal": "Community members'' user experience is improved in 6 months or less through the approval, funding, and implementation of impactful proposals.", "metrics": "I propose that Key Metrics to measure this Challenge Campaign''s success include:\n\n1.  The number of proposals submitted, to help measure proposer interest in the challenge.\n2.  The average community advisor proposal rating, to help measure the quality of proposals.\n3.  The number of proposals funded vs. the number of proposals that met the voting approval threshold. This can help measure if the funding allotment was sufficient for Fund 5. If this challenge campaign is renewed for future funds, this metric may help our community decide if the funding allotment needs to be increased or decreased.\n4.  The funded proposals'' **key performance indicators (KPIs**) overtime (the actual reported data, not only the definition of the KPI). This will help measure the effectiveness of the challenge overtime. See immediately below for more details regarding proposal KPIs.\n\nEach proposal is required to include at least one KPI that intends to measure impact on a specific user experience. User is defined as anyone who participates in Project Catalyst. Each proposal must describe:  \n\n1.  How the proposal intends to improve at least one specific user experience.\n2.  How the KPI intends to measure at least one specific user experience.\n3.  How the proposal intends to generate the KPI(s).\n4.  How the proposal intends to report the KPI(s) to our community.\n\nProposed Community Advisors scoring criteria for this Challenge Campaign (same as the current standard for evaluating proposals):  \n\n1.  Does this proposal effectively addresses the challenge?\n2.  Given experience and plan presented it is highly likely this proposal will be implemented successfully?\n3.  Does the proposal provide sufficient information to assess how feasible it is and how effectively it addresses the challenge?"}', -- extra
    'Kenny Keast', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    34,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=3), -- objective
    'ADA Epoch Clock',  -- title
    'How do we increase engagement, user experience, functionality, decentralization, and simplicity within the Cardano ecosystem?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=3), -- category - VITSS Compat ONLY
    'YW1rTUFjrE/ir5IaKR1nrVwuHYiEDBbkLPYcCx8HG5k=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBd0V', -- url
    '', -- files_url
    500, -- impact_score
    '{"brief": "The challenge is to create the ADA Epoch Clock. This clock will revolutionize our day to day activities and for billions that are currently unbanked provide a way to be autonomous and take control of finances, assets, and voting. The main challenge is to fit all the core components of the clock into a compact, durable, and lightweight form. Part of the challenge is internet connectivity. Each clock will act as a receiver and booster for its local area. Each clock will have very small solar panels that can also provide enough power to charge devices such as cell phones.\n\nHere are the features of the ADA Epoch Clock:\n\n\\* Standard Atomic Clock\n\n\\* Epoch Clock and Block Height\n\n\\* ADA Current Currency Price\n\n\\* Number of ADA in Circulation\n\n\\* At least one Photon Solar Cell (like Yanko Design or built in-house) or two or three to power clock and charge additional cell phones too - can have an AC plug to the wall too.\n\n\\* An appealing round orb with embossed Cardano logo and each of the six central circles has removable and upgradeable components to use as the blockchain evolves.\n\n\\* Each clock could also have a:\n\n\\- microstakepool - perhaps in microlaces 1/10 or 1/3 of a lovelace\n\n\\- Inbuilt Interoperable currency converter and tap payment function w inbuilt NFC tag reader\n\n\\- Inbuilt Daedalus wallet\n\nThese are some of the functional ideas for the ADA Epoch Clock.", "importance": "It is important for widespread adoption, ongoing network support, creating autonomy especially for the unbanked, and creating Cardano buzz!", "goal": "Success looks like unbanked people being able to use their one-stop-shop ADA Epoch Clock! A portal that provides time, payment, wifi & more!", "metrics": "There are many metrics to measure: Connectivity to the internet. Accurate readings of atomic time, ADA Epoch time, Block number, alternate time where the staking pool may be, and current price of ADA. Contactless payment usage. Hype and demand around the product. Energy production and use. Ease of use. Micro pool delegation, voting, and staking node. The way it has been seamlessly incorporated into everyday life as though it had always been there. Classic quality and a beautiful piece of art."}', -- extra
    'James Rees', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    35,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'African Digital Identity Platform',  -- title
    'Identity Management in the financial world has become a prerequisite, The steps associated with verifying identity leads to high wait rate',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'FDUyINpaQuDvwDVICJVkZZC9Tdu2B0scdswVMzL5w0o=', -- Public Payment Key
    '15000', -- funds
    'http://ideascale.com/t/UM5UZBd0Q', -- url
    '', -- files_url
    227, -- impact_score
    '{"solution": "By Creating multiple dapps that fits into the normal day to day activities of an average user, you onboard millions easily and seamlessly"}', -- extra
    'ankachain.vr', -- proposer name
    '', -- proposer contact
    'https://github.com/lawale4me', -- proposer URL
    'https://www.linkedin.com/in/lawale4me/', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    36,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Inu.Social - NFT Social Media',  -- title
    'Social media users/content creators are the product but disporporionately recieve less of the value generated due to the ad revenue model.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'CYaN32DSD/vMh/72HEQ7PFXXgLXJwVwG/swYIM49wuc=', -- Public Payment Key
    '100000', -- funds
    'http://ideascale.com/t/UM5UZBd0L', -- url
    '', -- files_url
    158, -- impact_score
    '{"solution": "Create a platform that uses Non-Fungible Tokens (NFT''s) to turn all content into collectibles and incentivise users to own/promote them."}', -- extra
    'Dan martino', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '\-Entrepreneur

\-Work in Corporate Advisory/Business Development', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    37,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Cardano Central',  -- title
    'No dedicated all encompassing social media sharing platform exists just for Cardano users.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'beZ2VYhX/Gi5nv2IzWJNQNnhe5iMS68pHC9Crx6Gt6I=', -- Public Payment Key
    '5000', -- funds
    'http://ideascale.com/t/UM5UZBd0I', -- url
    '', -- files_url
    165, -- impact_score
    '{"solution": "Cardano Central will be a custom and entirely unique social network 100% dedicated to the Cardano community."}', -- extra
    'matthewjones8', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Founded previous social media networks and experienced marketing executive.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    38,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Legitimatize blockchain solutions',  -- title
    'Massive user adoption of Cardano''s blockchain solutions will be obstructed in many countries due the lack of legislation related to crypto.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    'Iw86AnSWPgHvIEsjHoP50rBFOYbdCm3YD1Xr+Xq3mvQ=', -- Public Payment Key
    '1200', -- funds
    'http://ideascale.com/t/UM5UZBdzc', -- url
    '', -- files_url
    222, -- impact_score
    '{"solution": "We will bring juridical validity for blockchain transactions in Brazil, ensuring liability for Cardano''s DApps and developer ecosystem."}', -- extra
    'schappo', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Henrique has +10 years of experience as an entrepreneur in Brazil.

  

Filipe has +10 years experience with Brazilian judiciary system.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    39,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Virtual closet (to stake ADA)',  -- title
    'Staking, DeFi, NFT and many other terms from the blockchain space are foreign words for most people.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'Ws2sqozYgT9GVeFmNU48VPRJw5jobs7RB7rqbMZ89ZY=', -- Public Payment Key
    '44878', -- funds
    'http://ideascale.com/t/UM5UZBdzU', -- url
    '', -- files_url
    139, -- impact_score
    '{"solution": "Creation of an easy-to-use and understandable dApp version of a virtual closet where you can collect and stake (ADA) your purchased clothes."}', -- extra
    'Max', -- proposer name
    '', -- proposer contact
    'https://cryptellion.com/', -- proposer URL
    'Business Administration student (Focus on digitization)', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    40,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'ADA MakerSpace - DEV lessons',  -- title
    'Need new training videos for learning how to build with the Marlowe & Plutus playgrounds, some people learn better watching others do things',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    'DXJLwgJALO0od7vp2tW0Bvc6l1fkEN80p0Fheswoldg=', -- Public Payment Key
    '14800', -- funds
    'http://ideascale.com/t/UM5UZBdzQ', -- url
    '', -- files_url
    400, -- impact_score
    '{"solution": "ADA MakerSpace provides free video lessons for anyone interested in learning how to build things that will run on the Cardano blockchain"}', -- extra
    'Boone Bergsma', -- proposer name
    '', -- proposer contact
    'https://www.youtube.com/c/ADAMakerSpace', -- proposer URL
    'Since AUG192020 our channel has gotten 2,511 views, had Total Watch time of 258.4hours, and gained 225 Subscribers. 258hrs DEV lessons given', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    41,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Metadata oracle node',  -- title
    'Cardano metadata oracles are missing an operator-friendly node, making it hard for people to participate.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    'rb+OEAmchzDVqFsjl7BFY7q5DERC+WEQsRS/H/DEohk=', -- Public Payment Key
    '23000', -- funds
    'http://ideascale.com/t/UM5UZBdzJ', -- url
    '', -- files_url
    395, -- impact_score
    '{"solution": "We want to build an operator-friendly node software and would lower the barrier of entry for new oracle operators."}', -- extra
    'Marek Mahut', -- proposer name
    '', -- proposer contact
    'https://nut.link/', -- proposer URL
    'We launched the first metadata oracle on Cardano, and we were the first community public oracle pool on Ergo blockchain.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    42,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Fun competitive games with Cardano',  -- title
    'Introducing games into Cardano is a way to attract more users and more transactions into Cardano.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'gvNdgJy6buPc0n45OzgKxgtZ+BNICJh3jz0eeJx9LG4=', -- Public Payment Key
    '30000', -- funds
    'http://ideascale.com/t/UM5UZBdyj', -- url
    '', -- files_url
    167, -- impact_score
    '{"solution": "Create series of competitive games, starting with slither.io clone. Entry fee for players could be 1 to X ADA and winner takes all."}', -- extra
    'Ales Jiricek', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '15 years of programming experiences, 5 recent years with modern Typescript on big platforms such as managebystats.com or signageos.io', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    43,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Aregato - ADA ebook marketplace ',  -- title
    'Ebooks market is highly centralized, censored, and dominated by Amazon Kindle. There is a general lack of marketplaces in the crypto world',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    '/Wbz7i7wU8UvhsJy8WZTD2ByH+QM7jQ8SeCr2yR73pY=', -- Public Payment Key
    '7999', -- funds
    'http://ideascale.com/t/UM5UZBdye', -- url
    '', -- files_url
    167, -- impact_score
    '{"solution": "Aregato - ebook marketplace with blogging and social media features, built on top of Cardano, with Cardano native assets and NFTs"}', -- extra
    'Andrii Voloshyn', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Apart from 4+ years in blockchain (research, marketing, content) I wrote, published several books (in print, as ebooks) so I know this area', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    44,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Welcome to Cardano, developer!',  -- title
    'Too much information in a lot of places, sometimes difficult to find can discourage many developers. All with our support',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    'R19Gf5ZRVZAt5uPNYkL4iOYSMp+115hE2kiWSdevJIk=', -- Public Payment Key
    '25200', -- funds
    'http://ideascale.com/t/UM5UZBdyI', -- url
    '', -- files_url
    182, -- impact_score
    '{"solution": "We will create an updated signpost, a kind of developer portal, where all the information will be available."}', -- extra
    'Lukas Barta (Cardanians.io)', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'ITN, FF, marketing campaigns, web development', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    45,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'WeThinkItMatters, Cause-based Dapp',  -- title
    'Advertising is a $500+ Billion dollar a year industry, most of that money being spent to influence you is causing problems not solving them!',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'lFME8/gtqy99JgSnRGWYijsnzbRJkxHoUZwHM6aCTgs=', -- Public Payment Key
    '10000', -- funds
    'http://ideascale.com/t/UM5UZBdx4', -- url
    '', -- files_url
    240, -- impact_score
    '{"solution": "Cause-Integrated Advertising offers businesses a way to use marketing as a force for good and WTIM gives people power and influence over ADs"}', -- extra
    'Boone Bergsma', -- proposer name
    '', -- proposer contact
    'https://wethinkitmatters.com/', -- proposer URL
    'WeThinkItMatters is almost finished with V2 of our platform, have be paying DEVs already to work on our Cardano Smart Advertising Contract', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    46,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Smart Plug + Daedalus Wallet',  -- title
    'Venue owners lose money when power outlets are accessed without permission. They can absorb the cost, or attempt to restrict access.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'gLQc9C2ZOlSxXajt9AaQmqp4kH1QaIBPP0pT32ZW5mE=', -- Public Payment Key
    '20000', -- funds
    'http://ideascale.com/t/UM5UZBdxu', -- url
    '', -- files_url
    231, -- impact_score
    '{"solution": "Usage-based energy payment solution that enables venue owners to monetise power sockets located in public areas and shared accommodation."}', -- extra
    'Simon Montford', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Team of veteran domain experts with decades of experience working in tech startups and large enterprises; embedded systems, IoT, blockchain.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    47,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Orihon - Tracing Waste with Cardano',  -- title
    '2,120,000,000 Tons of waste was dumped on the planet and has a huge negative impact on the natural environment.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'GHr+6MVZHjKwrmolMNaDmh/ezN1RzFRSYUdHCDmSnrA=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBdxs', -- url
    '', -- files_url
    133, -- impact_score
    '{"solution": "We (Orihon) wish to create a Dapp that will trace waste through our Dapp and renumerate users for doing so."}', -- extra
    'Kenzo Guibeb', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Seven years in the IT Field as a Software and Systems Engineer. I have worked for Banks as a Business Analyst and Dassault Systems as Devops', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    48,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=3), -- objective
    'Cardano Ukraine ',  -- title
    'Promoting Cardano in RU\\UA speaking countries.',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=3), -- category - VITSS Compat ONLY
    '+JSRh/LLUBJ+6ehdUvtfMlpMn5rqaSW9KotqajfHe1Y=', -- Public Payment Key
    '6000', -- funds
    'http://ideascale.com/t/UM5UZBdxn', -- url
    '', -- files_url
    500, -- impact_score
    '{"brief": "We run a few local crypto-related chats and communities, where we can upload and represent the key information on the Cardano ecosystem.", "importance": "The Russian language is an official language in the 11 states, having 150 million native speakers.", "goal": "Success looking like a strong and happy community working together to onboard more and more users to the project they involved", "metrics": "A number of the Telegram users in local Cardano Telegram chat & channel, number of use-cases of ADA in the local real business."}', -- extra
    'Andrew', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    49,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Tatum.io - Cardano integration',  -- title
    'Wallet transactions are alpha omega. We have well-known platform tatum.io and we want bring Cardano to developers via our platform.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    'CEgxHzcgCDqEJ+ITfhTOUAiCO136AM2ooDiM4zfr/Aw=', -- Public Payment Key
    '15000', -- funds
    'http://ideascale.com/t/UM5UZBdxi', -- url
    '', -- files_url
    250, -- impact_score
    '{"solution": "We will implement full Cardano support into our platform - wallets, tokens, staking."}', -- extra
    'Lukas Barta (Cardanians.io)', -- proposer name
    '', -- proposer contact
    'https://github.com/tatumio/tatum', -- proposer URL
    'Fintech - Mastercard, Microsoft, Deloitte, Amazon', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    50,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Educational materials about Cardano',  -- title
    'As Cardano ecosystem grows, there is a constant need for educational materials in different languages.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    'C/l1s10D21/0zHClus3sQE3gHJLDZZx8Wjr8OitQepE=', -- Public Payment Key
    '15000', -- funds
    'http://ideascale.com/t/UM5UZBdxe', -- url
    '', -- files_url
    367, -- impact_score
    '{"solution": "Everstake is ready to create many educational materials about Cardano and distribute them among our customer base"}', -- extra
    'Everstake', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'We are Cardano SPO since Shelly launched, and we have experience with 30+ blockchains.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    51,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=3), -- objective
    'Privacy challenge',  -- title
    'How to improve Cardano ecosystem privacy?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=3), -- category - VITSS Compat ONLY
    'et0G/VOR+eJ8IWHlfCvSM5ZoTnQ0eIPUwG3Ga2j5hpM=', -- Public Payment Key
    '100000', -- funds
    'http://ideascale.com/t/UM5UZBdxb', -- url
    '', -- files_url
    500, -- impact_score
    '{"brief": "Cardano is lacking a lot of fundamental privacy features.\n\n  \n\nBy delegating, you are basically sharing all your addresses with the public.\n\nBy voting, you are basically sharing your catalyst decisions with the public.  \n\nBy running Daedalus, you are basically reveling your location to IOG.\n\n  \n\nI''m not talking about making Cardano a privacy coin, but there is a lot of features that could improvde privacy greatly. Cardano node over tor? Shielded ADA token? More research on Ouroboros Crypsinous?  \n\n  \n\nWhy we need privacy features in Cardano? discussion on Cardano forum: https://forum.cardano.org/t/why-we-need-privacy-features-in-cardano/35744", "importance": "Privacy gives us the power to choose what information we want to share publicly. Right now, Cardano is lacking privacy features.", "goal": "Winning proposals would greatly improve Cardano''s privacy proprieties.", "metrics": "Did the project improve Cardano''s ecosystem privacy features?"}', -- extra
    'Marek Mahut', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    52,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Funding portal (smart dApp)',  -- title
    'We will create a funding website, where users can create own fund requests, optionally benefits from that and smart contract will solve that',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'W7/tBoPn/z+lB2svrXhxEnJRlbyH6zTdoSm3Xyq+BBk=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBdxW', -- url
    '', -- files_url
    165, -- impact_score
    '{"solution": "We will create a funding website where users will be able to create their own applications for funding and a smart contract will solve it."}', -- extra
    'Lukas Barta (Cardanians.io)', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Community management, Itn, F&F, tools development', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    53,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Cardano Review Platform',  -- title
    'There are many proposals in each funding round. It can take a long time to sift through them individually and make an informed decision.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'iIhXVhi8/FVS1jzqMHvFClvX7WykTs9qqbuihS/qI/U=', -- Public Payment Key
    '10000', -- funds
    'http://ideascale.com/t/UM5UZBdxT', -- url
    '', -- files_url
    124, -- impact_score
    '{"solution": "We will provide detailed reviews and in-depth analysis of each proposal that we deem worthy of our investigation, with expertise insight."}', -- extra
    'matthewjones8', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'History of professional publication and a top writer, we have studied the blockchain industry for several years and are Cardano fanatics.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    54,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Localize Yoroi for Czech market',  -- title
    'Yoroi is missing Czech localization making it difficult to use on the local market.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    '9mrabL7/5+6PSelEm5hqNcHCgzwAv5xExvDCClbZTKA=', -- Public Payment Key
    '2300', -- funds
    'http://ideascale.com/t/UM5UZBdxP', -- url
    '', -- files_url
    378, -- impact_score
    '{"solution": "Provide Yoroi with a localized Czech interface. This needs to be up to date with each new Yoroi update and coordinated with translators."}', -- extra
    'Marek Mahut', -- proposer name
    '', -- proposer contact
    'https://www.fivebinaries.com/', -- proposer URL
    'We have been working on localization and translation of Signal Private Messenger, Fedora Project operating system or Secure Drop.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    55,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Localize Yoroi for Slovak market',  -- title
    'Yoroi is missing Slovak localization making it difficult to use on the local market.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    'S5fG+LdQ9rZq6y9+hjNZitm7qtPGlsoTEssBv5wlt4s=', -- Public Payment Key
    '2300', -- funds
    'http://ideascale.com/t/UM5UZBdxO', -- url
    '', -- files_url
    381, -- impact_score
    '{"solution": "Provide Yoroi with a localized Slovak interface. This needs to be up to date with each new Yoroi update and coordinated with translators."}', -- extra
    'Marek Mahut', -- proposer name
    '', -- proposer contact
    'https://www.fivebinaries.com/', -- proposer URL
    'We have been working on localization and translation of Signal Private Messenger, Fedora Project operating system or Secure Drop.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    56,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Cardano serialization library in Go',  -- title
    'Go programming language is currently missing many essential development libraries that would encourage people to develop on Cardano.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    'DJB4VP6Y52FkA1ycGjkSuOhwP/y2zpws2ITmFEIphfI=', -- Public Payment Key
    '6000', -- funds
    'http://ideascale.com/t/UM5UZBdxN', -- url
    '', -- files_url
    427, -- impact_score
    '{"solution": "We want to create Cardano serialization library for Go."}', -- extra
    'Marek Mahut', -- proposer name
    '', -- proposer contact
    'https://www.fivebinaries.com/', -- proposer URL
    'Among our members are seasoned programmers with vast blockchain programming experience, including Golang.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    57,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Implement CIP12 to Yoroi backends',  -- title
    'CIP12 defines on-chain stake pool operator to delegates communication and is missing implementation in Yoroi backend.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    'QC/vV9F609/mlc997TIpl0JstMJLt/ROfXA+Mdusuw0=', -- Public Payment Key
    '950', -- funds
    'http://ideascale.com/t/UM5UZBdxL', -- url
    '', -- files_url
    400, -- impact_score
    '{"solution": "We will implement CIP12 communications to Yoroi backend software."}', -- extra
    'Marek Mahut', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'We are co-authors of CIP12.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    58,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Metadata oracle endpoint in Yoroi',  -- title
    'Yoroi backend doesn''t have the capability for fetching metadata from oracle endpoints.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    'cCEkl5C2Ed46vuZWTw6JBD4YThMzRKse7rOL0jzB8Os=', -- Public Payment Key
    '1200', -- funds
    'http://ideascale.com/t/UM5UZBdxK', -- url
    '', -- files_url
    353, -- impact_score
    '{"solution": "We want to implement a new Yoroi backend endpoint that will provide the metadata oracles datapoints."}', -- extra
    'Marek Mahut', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'We launched the first metadata oracle on Cardano, and we were the first community public oracle pool on Ergo blockchain.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    59,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Metadata oracles explorer',  -- title
    'As the number of metadata oracles grows, we''re missing a single resource with overview and data about these oracles.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    'RWWpjFVKCLE85K3qSM6T7g6LHmiDTffxFHsnNwKul8Q=', -- Public Payment Key
    '6000', -- funds
    'http://ideascale.com/t/UM5UZBdxJ', -- url
    '', -- files_url
    382, -- impact_score
    '{"solution": "We would like to build a public metadata oracles explorer, that will serve as a go-to place for the developers to learn more about oracles."}', -- extra
    'Marek Mahut', -- proposer name
    '', -- proposer contact
    'https://nut.link/', -- proposer URL
    'We launched the first metadata oracle on Cardano, and we were the first community public oracle pool on Ergo blockchain.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    60,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Cardano Widgets to embed in DAPPs',  -- title
    'For a newbie in the crypto/DAPP ecosystem, it is difficult to obtain cryptos and maintain wallets etc.,',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    'VPREcHmFaqvefdrCERBqB+bEY8/pa5H/d7JxzWGAS1Q=', -- Public Payment Key
    '5000', -- funds
    'http://ideascale.com/t/UM5UZBdxG', -- url
    '', -- files_url
    150, -- impact_score
    '{"solution": "Help developers embed a widget in their DAPP page that enables easy FIAT on-ramp, account creation, transaction signing etc., for users"}', -- extra
    'Shri Raghu Raaman', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Ex-CoFounder, Blockchain Developer for 3 years, Ethereum and Cardano Trainer for 2 years and Full-Stack Developer, Crypto/ADA trader.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    61,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Lightweight KEVM Emulator',  -- title
    'Only EVM emulator exits, not KEVM emulator. Current EVM emulator is not instrumentable, slow and bulk. just a tool not framework.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    'VYki/wyMPUAvoNJ70ecx4nLuqnIy7j/RCzS+dV5TXEM=', -- Public Payment Key
    '20000', -- funds
    'http://ideascale.com/t/UM5UZBdwy', -- url
    '', -- files_url
    200, -- impact_score
    '{"solution": "We build a fully open source cross platform and multi arch lightweight framework since 2019. EVM and WASM module is in closed beta now."}', -- extra
    'kj', -- proposer name
    '', -- proposer contact
    'https://github.com/qilingframework/qiling', -- proposer URL
    'Qiling Framework is a open source cross platform and multi arch lightweight framework. It emulates windows, linux, macos on different arch.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    62,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'D Timestamped Registration System',  -- title
    'Patent registration with govt agencies is long, expensive and centralized process.  
DTRS wants to simplify, reduce costs and decentralize it.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'K7vuU5ZlYl6nrShNk3bvzqYFpc+4mPYK6D/HM6MhSOk=', -- Public Payment Key
    '5000', -- funds
    'http://ideascale.com/t/UM5UZBdwf', -- url
    '', -- files_url
    331, -- impact_score
    '{"solution": "DTRS is a simple and secure Dapp which gives as many people as possible the opportunity to register patents, ideas, etc. on the blockchain."}', -- extra
    'gwendal.ledivechen', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'We are a team of project managers, engineers and devs long involved in project development and tech industry and BC in Nigeria and Africa.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    63,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=3), -- objective
    'Cardano delegation & reward tracker',  -- title
    'Maximizing the profit for pool operators and delegators on basis of better insights.',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=3), -- category - VITSS Compat ONLY
    'YV9n7NvVs2lGY6Sjqwl4LMrZTboGoNq9zGqNZqCnV6M=', -- Public Payment Key
    '15000', -- funds
    'http://ideascale.com/t/UM5UZBdwJ', -- url
    '', -- files_url
    500, -- impact_score
    '{"brief": "We run a stake pool called ADA4Profit where we want to provide the best value to delegators.\n\n*   The solution will assist pool operators with insights and how they can improve their pool performance.\n*   The solution will assist delegators to find the best pool for maximizing their rewards on delegation.\n*   We plan to be open and transparent about what we do and what value we provide and aim to co-exist next to the other awesome solutions built by other community members.\n*   The solution will be web and app based\n\nFirst phase web based version, collect feedback from the community and improve.\n\nSecond phase mobile app version, collect feedback from the community and improve.\n\nWe think there is space for a better focused solution and this will also avoid monopolies in the community.", "importance": "Existing solutions are overwhelming with information about the whole ecosystem instead of focusing on the pool.", "goal": "We consider the solution successful in case we have acquired the majority of the stake pools (1000) and delegators using it on a daily basis", "metrics": "The success of the project can be measured by the number of downloads of the app, end-users in the app, the information being looked up and the searches being made by end-users."}', -- extra
    'Ron ADA4Profit', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    64,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Migrate Multi-PVP game from Tron',  -- title
    'Cardano needs easy-to-learn, multi-player games where ADA is used as the game currency and where large groups of members can play together',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    '8mhVmN2ufT37dh3848JMvv4uG2+EK/NJRnDJu1UfeWs=', -- Public Payment Key
    '40000', -- funds
    'http://ideascale.com/t/UM5UZBdv5', -- url
    '', -- files_url
    252, -- impact_score
    '{"solution": "Migrate existing successful Tron DApp to Cardano using KEVM Devnet.\n\nv1.0 to use ADA and Native Assets as game tokens inside smart contracts"}', -- extra
    'Ragnar Rex', -- proposer name
    '', -- proposer contact
    'https://www.traps.one/', -- proposer URL
    'Already top-20 Tron Dapp on DappRadar with high daily active users/transactions & active community  
Can bring users + community to Cardano', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    65,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Conversational UX/UI Toolkit',  -- title
    'The features of blockchain do not align with the features of our page-based interface design, this blocks the adoption of Cardano.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    'mOYLFVv92poMCPJe1TIx1knJRqiNXRza/t5d0lJwURQ=', -- Public Payment Key
    '36000', -- funds
    'http://ideascale.com/t/UM5UZBdv4', -- url
    '', -- files_url
    286, -- impact_score
    '{"solution": "The cUI toolkit speeds up time-to-market, removes dep. risk, aims for near real-time validation by offering conv. modules and scripts."}', -- extra
    'Niels Kijf', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Sr. Platform Designer. Four global e-commerce platforms (Selling) and training 36 Video Jockeys. (Education) = Conversational UI.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    66,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=3), -- objective
    'Sustainable Hubs in Cardano',  -- title
    'How we can create sustainable Hubs in Cardano?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=3), -- category - VITSS Compat ONLY
    'gSjWgafUDzUif/UvQqbMb4qtunvpmlJttMZBFcVbf8Q=', -- Public Payment Key
    '250000', -- funds
    'http://ideascale.com/t/UM5UZBdu2', -- url
    '', -- files_url
    500, -- impact_score
    '{"brief": "With the following growth in participation it will also start to become hard to follow these actors and keep track if these implementations are doing well, tracking KPIs etc.  \nThe thing is, a Cardano Hub can be many things, but how do we do a proper ranking and classification, how do we avoid bottlenecks to a quickstart too? I started to think in some categories, and these categories are able to interact in different levels, with different roles. At same time is interesting the possibility to explore the partnership between hubs to coordinate implementations.  \n\\-Community: All related to managing and empowering members  \n\\-Marketing: Communication is a very important aspect of any business  \n\\-Training: Development and training center  \n\\-Business: Participating, modeling and engaging in events looking for business opportunities  \n\\-Services: How to make the HUB less dependent from Catalyst  \nAll these segments are on top of Hub Administration capabilities.  \nEach Hub can have a level in each Service they want to provide, for example we can have a Level 1 hub in the Community category, this level will have certain aspects that will put him in this category, to change the level they need to prove capacity execution of a Level 2 Hub for the respective category, and maybe this will be achieved by collaborating with other hub in the same level. This will force other hubs to colab in order to achieve faster and better results.  \nBuilding a framework will require a lot of thought and will be a constant process, that is why we need a framework to think about these things and how we intend to achieve certain goals.  \n**250K is to fund many hub proposals under the hub framework, probably Level 1 Hubs.**\n\n**This proposal is too extensive, go to google docs.**\n\nhttp://bit.ly/hubdocs", "importance": "Our community can accelerate the ecosystem, in order to allow this to happen we need to find a sustainable way to deploy Hubs in Cardano.", "goal": "We will be able to provide a clear path on how to propose **sustainable Cardano Hubs**.", "metrics": "\\-We are able to create Hub Categories  \n\\-We are able to define success in each level of the hub activity.  \n(Ideas)  \n\\-We are able to create scenarios where partnering with other Hub to achieve a Level up goal represents a Score for the Hub."}', -- extra
    'MariaCarmo369', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    67,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Equipo para crear videojuego',  -- title
    'Hay muchas ideas sobre juegos y mucho talento en desarrolladores.

Debemos formar un equipo entre todos e incluir sus ideas en un gran juego',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'entK/OGyLindNj45xHgoCG+k7SPjVHuRWVZkmn0hiXw=', -- Public Payment Key
    '10000', -- funds
    'http://ideascale.com/t/UM5UZBdus', -- url
    '', -- files_url
    173, -- impact_score
    '{"solution": "RPG sobre la historia de la econom\u00eda, el jefe final es la Centralizaci\u00f3n del dinero Con recompensa en ADA y deben crear Daedalus para cobrar"}', -- extra
    'Daniel René Cabrera', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Ideólogo político, creativo, creador de contenido multimedia.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    68,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'StakeSync - DB Sync & Smash Add-on',  -- title
    'Developers & Stake pool operators need specific blockchain data often unavailable in existing tools or too intensive to compute on demand.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    'UdHtGrs3Ind8YjgpSpSqikBQt1G0Ws0jslB9lXgbUpk=', -- Public Payment Key
    '9000', -- funds
    'http://ideascale.com/t/UM5UZBdul', -- url
    '', -- files_url
    378, -- impact_score
    '{"solution": "StakeSync gathers & computes various live data (e.g. balances, delegations, pool metadata) from DB Sync and Smash into an extra database."}', -- extra
    'Nayeli Evans', -- proposer name
    '', -- proposer contact
    'https://stakesync.io', -- proposer URL
    'PHP & SQL development, Stake pool operation', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    69,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Cardano Sticker Marketing Campaign',  -- title
    'I want to raise awareness and adoption of Cardano by distributing many Cardano stickers (especially in East Europe).',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'V9oQTavqe08b1YrhLJ4E9lCmfXvmDvXEgtph1UFVR0E=', -- Public Payment Key
    '700', -- funds
    'http://ideascale.com/t/UM5UZBdub', -- url
    '', -- files_url
    321, -- impact_score
    '{"solution": "Cardano stickers can be a very cheap, but effective marketing method in my opinion."}', -- extra
    'Lorenzo Pietrapiana', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I Am 18 years old and follow Cardano closely since 2018. I love the community.Thats why I would love to contribute a bit to Cardanos success', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    70,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Cardano Role-based Access Control',  -- title
    'How can we enable decentralized organizations to share ownership and access of digital assets such as server resources?',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    'ZI8u0BCdqIzcQpRxBDXojUemnkkGU1EN+zUKKINCywE=', -- Public Payment Key
    '8000', -- funds
    'http://ideascale.com/t/UM5UZBduR', -- url
    '', -- files_url
    400, -- impact_score
    '{"solution": "Create a protocol and reference implementation defining a standard for creating and consuming RBAC policies on the Cardano blockchain."}', -- extra
    'Chad', -- proposer name
    '', -- proposer contact
    'https://github.com/torus-online/cardano-rbac', -- proposer URL
    'I''m a product manager and developer with experience designing, developing, and deploying enterprise SaaS B2B software.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    71,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Coinmarketcap for Dapps by Adapools',  -- title
    'We would like to create adadapps.org. It should be something like a coinmarketcap for decentralized applications in the Cardano ecosystem.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'uOza5f2peSBk+B8AzI1JyDiONB9nq3JJi72lcONZfYs=', -- Public Payment Key
    '40000', -- funds
    'http://ideascale.com/t/UM5UZBduH', -- url
    '', -- files_url
    200, -- impact_score
    '{"solution": "We will create awesome page with cool stuffs, open API and open source code. Idea is, in future can community devs contribute with code too."}', -- extra
    'ADApools.org', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '\- Part of ITN Friends & Family test.

\- Working for the community before the Cardano ITN.

\- Development of applications like ADApools.org.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    72,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'ADApools kit for developers, APIs',  -- title
    'We want to provide to developers a few basic tools in an open way to make it much easier to start developing on Cardano network.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    '0ApmIiV1tE+jPKdQmPLVrIbomya6zDTmzLgm+j3ylKE=', -- Public Payment Key
    '40000', -- funds
    'http://ideascale.com/t/UM5UZBduC', -- url
    '', -- files_url
    327, -- impact_score
    '{"solution": "We will provide several useful tools and extensions to developers and make their work easier."}', -- extra
    'ADApools.org', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '\- Part of ITN, Friends & Family test  

\- Working for the community since the Cardano ITN.

\- Development of applications like ADApools.org', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    73,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=3), -- objective
    'Cardano in Spanish',  -- title
    'How to solve the issue of lack of Spanish content for the Cardano Project?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=3), -- category - VITSS Compat ONLY
    'mhEyOpzVn7gqTFjvzn/1V+UqrBU2vuC096mx/ZNDdAo=', -- Public Payment Key
    '6000', -- funds
    'http://ideascale.com/t/UM5UZBdt6', -- url
    '', -- files_url
    500, -- impact_score
    '{"brief": "Hispanic community consists of +20 countries. Providing Cardano Spanish content is relevant not only for the current community but also for all the future Hispanic members that will be onboarding the Cardano ecosystem, content will be already available for them in the years to come. Making a stronger Hispanic Cardano community means making a stronger worldwide Cardano community.\n\n  \n\n**VERSI\u00d3N DE LA PROPUESTA EN ESPA\u00d1OL:**\n\n**T\u00edtulo del Desaf\u00edo**\n\nCardano en Espa\u00f1ol\n\n**Pregunta del desaf\u00edo**\n\n\u00bfC\u00f3mo resolver el problema de falta de contenido en espa\u00f1ol para el Proyecto Cardano?\n\n**\u00bfPor qu\u00e9 es importante?**\n\nLa comunidad hispana Cardano es la segunda en tama\u00f1o y est\u00e1 creciendo r\u00e1pidamente. Sin embargo, a\u00fan no se ha publicado ning\u00fan contenido oficial en espa\u00f1ol.\n\n**\u00bfC\u00f3mo se ve el \u00e9xito?**\n\nCrecer la comunidad hispana de Cardano\n\n**M\u00e9tricas clave para medir**\n\n*   N\u00famero de suscriptores a canales de Youtube en espa\u00f1ol\n*   N\u00famero de visitas\n*   N\u00famero de horas de reproducci\u00f3n\n*   N\u00famero de suscriptores de los canales oficiales y comunitarios de Telegram hispanos\n*   N\u00famero de vistas para el contenido en espa\u00f1ol en el Foro de Cardano", "importance": "Hispanic Cardano community is the second in size and growing fast. Nevertheless, no official Spanish content is yet being published.", "goal": "Grow the Cardano Hispanic community through educational content material.", "metrics": "*   Number of subscribers to the hispanic Youtube channels\n*   Number of visits\n*   Number of watched hours\n*   Number of subscribers to the official and community driven Hispanic Telegram channels\n*   Number of views for Spanish content in the Cardano Forum"}', -- extra
    'Seba (Spanish Translator)', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    74,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Multiplayer strategy game',  -- title
    'Increase adoption',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'YxMO/j1LdsEmeCQiqY/0ECe2cKxbmfwcFB1Tx3tPBzc=', -- Public Payment Key
    '2500', -- funds
    'http://ideascale.com/t/UM5UZBdt4', -- url
    '', -- files_url
    296, -- impact_score
    '{"solution": "Build a mobile/desktop game that does not require a blockchain but adds additional features to the game which will be exposed to the users."}', -- extra
    'Simon Schubert', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '9+ years of experience in mobile development', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    75,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Dapp for business contracts',  -- title
    'Making business contracts legally valid and bound is not affordable for everyone especially in developing countries.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'UOOsp5v1LLnGigQ53EyHH0cli/EwDO9BCoSZD4J6PpE=', -- Public Payment Key
    '15000', -- funds
    'http://ideascale.com/t/UM5UZBdt1', -- url
    '', -- files_url
    317, -- impact_score
    '{"solution": "Create an easy to use app for widely available mobile phones where people can upload PDF contacts and sign or verify them."}', -- extra
    'Ron ADA4Profit', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'We have previous built & deployed tokenization solutions (in Ethereum) with digital signing and and mobile wallet app.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    76,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=3), -- objective
    'More marketing for ADA',  -- title
    'How many normal people (outside crypto world) have heard of ADA?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=3), -- category - VITSS Compat ONLY
    'L5geziq6j0j7+47W3CxyyNg7tds8sXhi5Dls+MLL3JY=', -- Public Payment Key
    '300000', -- funds
    'http://ideascale.com/t/UM5UZBdtY', -- url
    '', -- files_url
    500, -- impact_score
    '{"brief": "CARDANO in everyone''s ears.", "importance": "The more people hearing and knowing what Cardano project means, the more people will investi in Cardano, in ADA.", "goal": "Changing people''s life in a good way.", "metrics": "The number of people the information would reach.\n\nPromoting ADA through an influential and trustworthy person."}', -- extra
    'Delia.ciurea2013', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    77,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=3), -- objective
    'Cardano Sticker Marketing Campaign',  -- title
    'Hi everybody,

I will travel to East Europe this year and would like stick on as many Cardano stickers in different places as possible.',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=3), -- category - VITSS Compat ONLY
    'OMJ989YV9tylSvBeV/cXPnAghZ/tHUUPLsGyN6kpFl0=', -- Public Payment Key
    '600', -- funds
    'http://ideascale.com/t/UM5UZBdtF', -- url
    '', -- files_url
    500, -- impact_score
    '{"brief": "I would probably print a very basic Cardano logo with the name \"Cardano\" besides it. Like on the Cardano Wikipedia page. So everyone will understand it and not knowing what it means will hopefully spark curiosity...\n\nAlso, I would make a video where you can see the Cardano stickers in some cool (and famous) locations and photograph all affixed stickers, so you know that I have actually done the job :)\n\n  \n\nThanks a lot for reading :)\n\nBy the way, I am 18 years old and have been following Cardano very closely since beginning of 2018. I just love the vision and the community. That''s why I would love to contribute a little bit to Cardanos success :)", "importance": "It''s important because a lot of people will see Cardano stickers and eventually research it. Especially in East Europe adoption is needed.", "goal": "It''s hard to measure success here, but hundreds of thousands of people will see the Cardano logo regularly which could lead to more adoption", "metrics": "How many people will see the stickers? I would buy 2000-3000 stickers and affix them in all sorts of cities. Not only in East Europe, but also in my hometown Hamburg, Germany; I also know lots of people travelling, who would be willing to help me stick them on in different parts of the world. I would affix them in areas where many people live and also where it''s relevant (for example before banks or in crowded areas).\n\nVery roughly (and probably even conservatively) maybe 100 to 200 different people will see one sticker; With 3000 stickers that would be 300.000-600.000 people. Let''s just say that 1% will be interested in Cardano after doing some fast Google research. That would still be roughly 5.000 people. And again, I think these numbers will probably even be much higher.\n\nWhy East Europe is good for some more marketing: a lot of people are underbanked there and would probably find Cardanos ideas very interesting (and hopefully also practical when DEFI on Cardano is possible)."}', -- extra
    'Lorenzo Pietrapiana', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    78,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Adding Cardano to TOP Game Dapp',  -- title
    'One of the biggest problems in the current Cardano ecosystem is the unavailability of dapps use cases. We can change that!',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'tAajICQq9Db12KF7RQwCacv4oYxvap6TUZJVbOWjK+k=', -- Public Payment Key
    '10000', -- funds
    'http://ideascale.com/t/UM5UZBdtE', -- url
    '', -- files_url
    173, -- impact_score
    '{"solution": "We propose adding Cardano to the CryptoBrewMaster as a Login method, token swap possibility, NFT creation and swap, and so on"}', -- extra
    'Andrew', -- proposer name
    '', -- proposer contact
    'https://www.cryptobrewmaster.io/home', -- proposer URL
    'We are a team of developers, designers, and marketing experts working on CryptoBrewMaster. CBM is currently a top 10 DAPPradar game dapps', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    79,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Fanance -Celebrity Trading Platform',  -- title
    'Many people hesitate to invest in the stock/crypto market due to a lack of knowledge about the market/stocks. Lots of research is required',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'DXr6dcRxU7n56TRV3ThnUbMprqZNJUkcIjd09QB1bRg=', -- Public Payment Key
    '20000', -- funds
    'http://ideascale.com/t/UM5UZBds8', -- url
    '', -- files_url
    250, -- impact_score
    '{"solution": "Fanance club is a trading platform where one can capitalize their existing sports knowledge & invest in their favourite Sports Stars"}', -- extra
    'Fanance Club', -- proposer name
    '', -- proposer contact
    'https://fanance.club', -- proposer URL
    'Ex-CoFounder, Blockchain Developer for 3 years, Ethereum and Cardano Trainer for 2 years and Full-Stack Developer, Crypto/ADA trader.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    80,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'OpenAPI Integration: Ebay & Shopify',  -- title
    'Selling platforms put so much fees on top of sold products & drives Entrepreneurs to quit selling. Online sellers are paying too much fees.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'wn55Bvg+BcNqp7Bt6HnP5ET/xsPzcU1Wmsmw54rlnU0=', -- Public Payment Key
    '40000', -- funds
    'http://ideascale.com/t/UM5UZBdsy', -- url
    '', -- files_url
    173, -- impact_score
    '{"solution": "Create a DApp/API integration maintained on the blockchain to expose Cardano to quality markets and customers that will drive mass adoption."}', -- extra
    'Rey Villena Jr.', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '2 years of experience on this field & Ive been running online business for a while now & we need implementers for this life changing project', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    81,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Write Dapps as continuous workflows',  -- title
    'DApp developpers maintainers and verifiers face the "endpoint hell" in Web, in chain and in out-chain code',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    'SzS0MIHdZH64Q1rFKjl6P2D5js+YYGITRmxOzM+occM=', -- Public Payment Key
    '7000', -- funds
    'http://ideascale.com/t/UM5UZBdsv', -- url
    '', -- files_url
    400, -- impact_score
    '{"solution": "Express the entire program flow within a Haskell monad which set HTTP endpoints for microservices and uses Plutus endpoints"}', -- extra
    'agocorona', -- proposer name
    '', -- proposer contact
    'https://github.com/agocorona', -- proposer URL
    'Program in Haskell since the year 2005

Professional experience in C,C++,Java, JavaScript, C#,

Author of similar integration libraries', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    82,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=3), -- objective
    'DeFi/CeFi Cardano&...TBC in Fund 5',  -- title
    'How can we create an environment for seamless (no fee/delay) integration b/w ADA and FIAT?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=3), -- category - VITSS Compat ONLY
    '3LscBxPBAu0i8D+01gXgHei4UziUvH2PDEMzFAExTUQ=', -- Public Payment Key
    '400000', -- funds
    'http://ideascale.com/t/UM5UZBdsr', -- url
    '', -- files_url
    500, -- impact_score
    '{"brief": "I want to clarify that \"How can we create an environment for seamless (no fee/delay) integration b/w ADA and FIAT?\" is the Community Choice challenge up for vote in Fund 3.\n\nWhy do we need this type of environment? To become a World Financial OS. THIS is how we change the world.\n\nI have a solution, which will be given in Fund 5.", "importance": "For Cardano to become a World FinOS, the integration b/w ADA and FIAT must be seamless.", "goal": "Wide user adoption of ADA, Spending ADA at stores, broader audience served.", "metrics": "Monthly user transaction volumes, Number of users onboarded monthly, number of accounts, number of international transactions issued monthly, monthly rewards collected,"}', -- extra
    'drakemonroe', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    83,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Visual Studio Smart Contract Plugin',  -- title
    'There are no plugins within Microsoft Visual Studio to build Cardano Smart Contracts on vb or c# .NET. VS is used by millions of developers',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    'eeD4iWa8oKFxlTzjv4KOuhiVg2sWL0VhiFDy3wU4Er4=', -- Public Payment Key
    '16450', -- funds
    'http://ideascale.com/t/UM5UZBdsk', -- url
    '', -- files_url
    468, -- impact_score
    '{"solution": "The plugin will simplify how you create Smart Contracts on Cardano by creating a template project and instructions on how to get started"}', -- extra
    'Rob Greig', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I have been developing on Visual Studio for over 20 years and have built many plugins and extensions for the development tool.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    84,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Cardano for Drupal developers',  -- title
    'There are currently no modules or tools that the Drupal community can use to implement Cardano blockchain solutions.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    '1J8ZtRVGog5vpuMwg5L3+4RypmG4u2ucSP2lQSCz6F8=', -- Public Payment Key
    '48000', -- funds
    'http://ideascale.com/t/UM5UZBdsU', -- url
    '', -- files_url
    192, -- impact_score
    '{"solution": "We would like to develop basic open source Drupal modules that are required for developing Cardano blockchain solutions on Drupal."}', -- extra
    'Kakw', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'We have proven experience with Drupal, PHP and other open source technologies.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    85,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Automation platform integration',  -- title
    'Blockchain and traditional tools should not compete. Bringing blockchain in automation tools like IFTT or Zapier can encourage developers.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    'gYATxzv3kw0X8IXvjORCIR3Bg2f1xST8v9raDk4a/Tk=', -- Public Payment Key
    '30000', -- funds
    'http://ideascale.com/t/UM5UZBdsP', -- url
    '', -- files_url
    156, -- impact_score
    '{"solution": "Tools to integrate event listening on top of contract address to trigger automation"}', -- extra
    'Jean', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I''m senior developer using automation tools everyday.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    86,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=3), -- objective
    'Cardano Blockchain Strategy Game',  -- title
    '++The intention is to create a game, learn basic concepts of the blockchain, living in a decentralized world. See descriptive diagram, annex.++',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=3), -- category - VITSS Compat ONLY
    'KGIm2w2hl/7IYuZ75SF9z8Ewy9uzxgYCVdHOAjn5TOY=', -- Public Payment Key
    '30000', -- funds
    'http://ideascale.com/t/UM5UZBdsF', -- url
    '', -- files_url
    500, -- impact_score
    '{"brief": "PROBLEM STATEMENT:\n\nIn order to onboard new community members and for the ecosystem to continuously expand, future Cardano users require continual education to understand the use cryptocurrency and how it can improve their financial and social well-being.\n\nDESCRIBE YOUR SOLUTION TO THE PROBLEM:\n\nPeople learn through play, therefore the intention is to create a game in which people can learn the basics of decentralization, using cryptocurrencies and living in a decentralized world.\n\nRELEVANT EXPERIENCE:\n\n\u2022 The team will require funding to attract developers experienced in development of gaming platforms, particularly android developers\n\nGAME BASICS\n\nThe idea is to do this through a game similar to SimCity. Here everyone''s life would be based on using Cardano as the main currency, including making payments in ADA. Within the game everything related to personal finances would be based on Cardano and the Daedalus/Yoroi wallets.\n\n\u2022 In the different community businesses, you pay with ADA.\n\n\u2022 Shopping for groceries, clothes by using ADA wallets such as Daedalus or Yoroi\n\n\u2022 Applying for University, getting your degree. Using Cardano Prism for decentralized Identity Management.\n\n\u2022 Getting a job and being paid for that job in $ADA. Proving your education credentials with Prism.\n\n\uf0d8 SKILL TREE:\n\nA person wants to see the different options of existing careers, and decides to explore what, how and where his Employment Contract would be developed, after graduating.\n\nThe person chooses the job. She begins to work in the position of General Services, so she begins to know the company. To be promoted to the next position, according to the structure of the company, you must comply with and exceed actions that help improve the company.\n\nAs you go up, you will unlock in a Skill Tree, The different skills to unlock more skills and keep expanding the company further.\n\nRecording skills and certifications on the Cardano blockchain\n\n\u2022 Applying for personal financial products such as vehicle loans or buying a home with a loan via the Cardano Blockchain.\n\n\u2022 Paying taxes, travelling, sending money to others\n\n\u2022 Collecting items of value such as art, jewelry with authenticity registered on the blockchain.\n\n\u2022 Having valuables stolen (art, jewelery, land) and helping the authorities recover the valuables by proving ownership and authenticity on the blockchain\n\n\u2022 Participating in community governance, using blockchain based voting apps on the Cardano blockchain.\n\n\u2022 Build on social policy with the game rewarding actions that improve decentralization", "importance": "Future ADA users require education to understand financial use to improve their social welfare and financed.", "goal": "A Community in constant growth", "metrics": "Minimum Viable Product:\n\n1\\. Mobile based platform\n\n2\\. Decentralized SimCommunity\n\n3\\. Basic cryptocurrency actions: sending, receiving, purchasing, storing, assigning unique identifiers, etc.\n\n4\\. Roles: Players, shop owners, police, doctors, city administrators, etc.\n\n5\\. Mechanics of the game, levels for progression, rewards mechanism, etc."}', -- extra
    'Maritza J Marquez', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    87,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Decentralized Social Media Platform',  -- title
    'People should be able to speak freely on the internet. Centralized services like Twitter control what content can be shared and by whom.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'sCPbiHlbiXkHmFeWlHAV+6xRHHFQ6pDepb/BjnW9E8U=', -- Public Payment Key
    '100000', -- funds
    'http://ideascale.com/t/UM5UZBdsA', -- url
    '', -- files_url
    192, -- impact_score
    '{"solution": "Create a decentralized, public, permission-less social media platform that ensures freedom of speech and data sovereignty."}', -- extra
    'Kodex Data', -- proposer name
    '', -- proposer contact
    'https://github.com/kodexdata', -- proposer URL
    'Full stack engineers

Smart Contract developers

Social media savants', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    88,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=3), -- objective
    'Social media promotion',  -- title
    'How can we promote Cardano and include the young generation to the network?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=3), -- category - VITSS Compat ONLY
    '4RJcJPDThThwnQww0OGT3kXunIxZ725bVHHpV5uOxtk=', -- Public Payment Key
    '20160', -- funds
    'http://ideascale.com/t/UM5UZBdr5', -- url
    '', -- files_url
    500, -- impact_score
    '{"brief": "This project consists in 4 strategies to promote Cardano''s network in mainstream social media networks in order to attract the young generation resulting in visibility, wallet downloads, usage and education. Also Developers and Entrepreneurs might get interested in building on Cardano because of indirectly being exposed to mainstream social media promotion.  \n  \nInstagram strategy (Female focus):  \nContact of several emergent models (10K to 100K followers). Then, propose the following: exchange of downloading the wallet and advertise it in their content (model downloading the wallet and hashtags) versus giving them access to our community. Then create a website with the list of models that downloaded the wallet and share it with our community.  \nGive the models some visibility in our community by promoting them in Cardano youtube channels and Cardano youtubers that are willing to support them.  \nOur community can support them by following, liking, writing messages and giving donations.  \n  \n\nTik Tok strategy (young male/female focus):\n\nCreation of engaging content related to Cardano and crypto\n\nCreation of trending content not crypto related adding but Cardano, ADA and crypto tags\n\nGaming Youtube (Male focus):  \nCreation of engaging and trending (highlights and funny moments) 2 to 4min edited video of several amateur gamers that have a growing follower amount and put Cardano Logo, Wallet and info as a sponsor/add  \nAsk to mentioned gamers to share our video  \nGive the gamers some visibility in our community by promoting them in Cardano youtube channels and Cardano youtubers that are willing to support them.  \n  \n\nMusic Youtube Playlist (General focus)\n\nCreation of playlists modern trending with futuristic tendency in order to expose the general public the logo and name of Cardano. The channel would be called Cardano beats or something along those lines.\n\n  \n\nStaff needed:\n\n1 Project manager - Previous management experience to fulfill this role.\n\n1 Assistant manager - Previous management, supervisor, assistant, or senior experience to fulfill this role.\n\n15 Promoters - 5 Tik Tok, 5 Instagram, 5 Youtube (3 hours per day each) (several languages). Being familiar with mentioned platforms is enough to fulfill this role.\n\n1 Web site manager - Wix, Squarespace, Google sites or Wordpress previous experience is needed to fulfill this role.\n\nPlan:\n\n\\* Project manager communicates the plan by video conference to the team.\n\n\\* Creation of 1 web site divided in 3 main URLs. 1) ADA Models 2) ADA Tik Tok influencers 3) ADA Gaming and Music Youtube.\n\nThe web sites will be simple and will display a list of around 50 influencers per page.\n\n\\*Target: emergent influencers with 5K to 10K followers and influencers with 10K to 500K followers\n\n\\*Proposition to influencers: Advertisement in form of 1) Posts (Instagram), 2) Funny and entertaining 10 to 15 sec video content promotion (Tik Tok) 3) 15 sec sponsor ad in Gaming Youtube channels 4) Youtube Music playlists. In exchange we post their name in our site web of influencers and share it with our community.\n\nExamples of Cardano exposure content.\n\n1) Instagram - 112K followers - South Korean Athlete and Model https://www.instagram.com/run.soyoung/?hl=en\n\n2) Tik Tok - 59.3K followers - Emergent influencer with Funny content https://www.tiktok.com/@rocyaeardrumz?lang=en\n\n3) Youtube Gaming - E-sports gamer with loyal community https://www.youtube.com/channel/UCNEmy4a6O2q0ZCz7Qi2MThA\n\n4) Youtube Music playlist - showing Cardano logo and important info pop ups, themes futuristic, modern, chill, electronic, upbeat, programming, hacking, dubstep, trap, edm, gaming, trance, progressive, house, lounge, chillstep OR retro-beat 2021 mix\n\nhttps://www.youtube.com/watch?v=l9nh1l8ZIJQ  \n\nhttps://www.youtube.com/watch?v=EH1I-8KyI9Y\n\n\\*Promoters contact influencers sending waves of pre-made messages: Monday, Tuesday, Wednesday and Thursday for 3 hours per day. Total of 180 hrs per week (15 promoters x 4 days x 3hrs per day) of messages follow ups included\n\n\\*Influencers interested will reply.\n\n\\*Follow up on the offer and set date of the ad. Ask them to advertise the download of the Daedalus or Yoroi wallet showing the action in their own style. Ask them to add a description and Cardano #hashtags ADA, Daedalus, Yoroi, Wallet, Crypto, Moon, ATH, BTC, Bitcoin, buildincardano.\n\n\\*Add the influencers in our web site list.\n\n\\*Assistant manager and Manager share web site with the Cardano community: Youtube Channels that promote Cardano, Catalyst members, CF members, Cardano Twitter, Discord and Telegram.\n\n  \n\nDetail budget:\n\nPromoters - 180 per week x 6 weeks = 1080 x $15 per hour = $16,200\n\nExpected result:\n\n\\- Instagram: 24 messages per 2hrs + 1hr of research and follow ups per promoter (x5 promoters) = 120 influencers reach per day x 4 = 480 per week x 6 weeks = 2880 in total\n\n\\- Tik Tok: 8 messages per 2hrs + 1hr of research and follow ups per promoter (x5 promoters) = 40 influencers reach per day x 4 = 160 per week x 6 weeks = 960 in total\n\n\\- Gaming Youtubers: 8 messages per 2hrs + 1hr of research and follow ups per promoter (x4 promoters) = 32 influencers reach per day x 4 = 128 per week x 6 weeks = 768 in total\n\n\\- Music Youtube: 1hr research + 1:30hrs take list of artists and ask permission to use their music + 3hrs to create the playlist + and 30 min to public on Youtube the playlist = 1 playlist per 2 days = 2 playlists per week x 6 = 12 playlists\n\n  \n\nWeb site creation and management - $15 per hr x 4hrs per day = $60 x 4 days per week = $240 per week x per 6 weeks = $1440\n\n  \n\nAssistant Manager - Creation of influencer lists: 1) Instagram list 2) Tik Tok list 3) Gaming Youtube list 4) Music Youtube list\n\nCommunicating with the Project Manager and with the Promoters, create daily video or chat meetings, track progress, gather and transfer the data from the promoters to the website manager\n\n$15 per hr x 3hrs per day = $45 x 4 days per week = $180 per week x per 6 weeks = $1080\n\n  \n\nProject Manager - Lead the team, give direction to the project, track daily and weekly progress, communicate with every member of the promoter team at least twice a week, communicate with the assistant manager on a daily basis, communicate with the website manager once or twice a week, follow up website development, create the pre-made messages for each campain, give advice to team members on how to follow up, communicate progress with Catalyst weekly.\n\n$15 per hr x 4hrs per day = $60 x 4 days per week = $240 per week x per 6 weeks = $1440  \n\n  \n\nTotal cost of the project: 20,160", "importance": "Millenials and Gen Z will change the world and will empower this technology and the blockchain era more than anybody.", "goal": "Young people download and uses Daedalus after being exposed to Cardano''s promotion on Tik Tok, Instagram and Gaming Youtube", "metrics": "Some statistics:  \nTik Tok  \nhttps://www.omnicoreagency.com/tiktok-statistics/  \nInstagram  \nhttps://www.statista.com/statistics/578364/countries-with-most-instagram-users/  \nYoutube  \nhttps://www.thinkwithgoogle.com/marketing-strategies/video/statistics-youtube-gaming-content/"}', -- extra
    'Jonas Iniguez', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    89,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Python module',  -- title
    'Right now there is a plain REST API available for communicating with the wallet but no higher-level abstraction classes for Python.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    'mDr430zUWxMLfnt0/uNX9Z/dZO5Y4N8Q2XnJPEoNV1E=', -- Public Payment Key
    '10000', -- funds
    'http://ideascale.com/t/UM5UZBdro', -- url
    '', -- files_url
    474, -- impact_score
    '{"solution": "Create a Python module that implements base classes for Wallet, Address, Key, Transaction, etc. and offers well-structured exception tree."}', -- extra
    'emes', -- proposer name
    '', -- proposer contact
    'https://github.com/emesik', -- proposer URL
    'I''m experienced Python developer and the main author of a similar module for Monero: https://github.com/monero-ecosystem/monero-python', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    90,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=3), -- objective
    'Cardano Website Localization',  -- title
    'How to get the wallet downloaded around the world and meet a global community localization standards?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=3), -- category - VITSS Compat ONLY
    '9L5OCMllED3OsQ3zxTmQ1xU/xWAURx/NFpbSZHVjBhs=', -- Public Payment Key
    '26730', -- funds
    'http://ideascale.com/t/UM5UZBdra', -- url
    '', -- files_url
    500, -- impact_score
    '{"brief": "Localize the Cardano and Daedalus Wallet Content in 32 more languages so developers and users around the world have easy access to the product:  \nhttps://cardano.org/  \nhttps://daedaluswallet.io/  \nKorean  \nFrench  \nSpanish  \nItalian  \nPortuguese  \nGerman  \nDutch  \nNorwegian  \nSwedish  \nDanish  \nFinnish  \nRussian  \nUkrainian  \nPolish  \nCzech  \nBulgarian  \nRomanian  \nGreek  \nTurkish  \nHebrew  \nHindi  \nBengali  \nPunjabi  \nArabic  \nUrdu  \nPersian  \nTamil  \nVietnamese  \nThai  \nIndonesian  \nSimplified Chinese  \nCantonese  \nTraditional Chinese  \n  \n\nProjects examples with competitive localization: Ethereum, Binance chain, Solana  \n  \n\nPlan:  \nProject Manager explains the plan of the project through video conferences after the community proposals are submitted and chosen.  \nProject Manager makes first contact with Website Manager  \nCoordinator create the Localization progress list, create the translator and the website templates, he do research on Upwork and Fiver (and LinkedIn or other if needed) and create a list of translators.  \nCoordinator gives the assigned translator''s contact and templates to the respective 32 Language owners.  \nLanguage Owners contact their assigned translator and gives content, the translator template, set dates of delivery and add them to the Localization Progress list.  \nProject Manager tracks project progress and updates data based on progress.  \nProject Manager communicates progress to Catalyst  \nLanguage Owners receives translated content, verifies it and fill the website template based on the translation.  \nLanguage Owners give website localization to the Coordinator  \nCoordinator gather the 32 languages website templates  \nProject manager verifies and passes the localizations to the Website Manager  \nWebsite Manager add the localizations to the respective URLs  \n  \n\nStaff and tasks:  \nProject manager: 1) Direct the project 2) communicate with all team members 3) communicate with Catalyst and Cardano website manager 4) track progress 5) gather all the information. Previous management work experience is needed to fulfill this role.  \nCoordinator: Person in charge of 1) creating the template for translator and template for website manager. 2) Coordinating the language owners content. 3) Research of translators in Upwork, Fiverr, Catalyst or other. Previous supervisor, management, coordinator or senior position is needed for this role.  \nLanguage owner: Person in charge of 1) Hire the chosen translator, communicate, give, receive and verify translated content. 2) Organize the translation in the template for website manager. The Language owner have to be proficient in one of the languages that has to be localized as well as English.  \n  \n\nNote:  \n\\- If a someone has translation experience or degree then he/she can present his LinkedIn and submit for translator, this would mean he/she would take both: the tasks, responsibilities and rewards of language owner and translator.  \n\\- The following URLs information would not be translated in this fund due to large amount of text vs lower visibility ratio: Cardano Docs, Research and Developer updates within the Developers button.  \n  \n\nTo prove proficiency:  \n\\* Localized language -  \na) Is needed, only a statement that the language to localize is the native language or  \nb) some proof of studies that a the 2nd or 3rd language is spoken and written with proficiency (e.g. college degree in a foreign institution, language or translation degree or a degree in a language institution)  \n\\* English proficiency -  \nsome proof that studies have been made in English (LinkedIn preferable).  \n  \n\nDetailed Budget:  \n  \n\n\\*Translation: $30 per page per language (500 words)  \ncardano.org = 11 pages = $330  \ndaedaluswallet.io = 1 page = $30  \nTotal per language translation = $360  \n  \n\n\\* Language owner:  \nTeam video conference = 1h30 = $30  \nHiring and initial communication = 1hr = $20  \nGive English content and explain details = 1hr = $20  \nFollow progress = 1hr = $20 (divided it by 4 x 15min. if needed)  \nReceive and verify = 30 min. per page x 12 pages = 6hrs = $120  \nAdd the content to the website template = 6hrs = $120  \nCommunicate with coordinator = 1hr = $20  \nTotal per language owner = $350\n\nTotal per language: translation $360 + owner $350 = $710  \nTotal language cost: $710 x 32 languages = $22,720  \n  \n\n\\-Management and coordination  \n  \n\n\\*Coordinator:  \nCreate team list of contacts and add Name Language and email and dates of Localization Progress of every member = 2hrs = $40  \nTeam video conference = 1h30 x 6 = $180  \nTranslator templates: 3hrs = $60  \nWebsite templates: 3hrs = $60  \nCommunicate with language owner = 1hr x 32 = $640  \nOrganize received templates: 2hrs = $40  \nCommunicate with Project manager = 4hrs = $80  \nTotal for coordination: $1100  \n  \n\n\\*Project Manager:  \nCreates video conference schedule and sends invite to all members = 1hr =20$  \nTeam video conference = 1h30 x 6 = $180  \nCommunicate with web site manager = 4hrs = $80  \nGather project progress info and communicate with Catalyst weekly meeting = 6hrs = $120  \nCommunicate with coordinator = 4hrs = $80  \nFollow ups on team members = 2hrs x 32 = $1280  \nUpdates continuously the Localization Progress List = 6hr = $120  \nTotal for management: $1880  \n  \n\n\\-Unexpected fees budget:  \n1 language cost = $710  \nextra coordination 8hrs = $160  \nextra management 8hrs = $160  \nTotal unexpected fees budget = $1030  \n  \n\nTotal cost of the project:  \n$26,730", "importance": "Mass adoption.", "goal": "Cardano and Daedalus Wallet Website content is Localized at least as much as Cardano''s competition (e.g. Ethereum localized in 32 languages)", "metrics": "https://ethereum.org/en/languages/"}', -- extra
    'Jonas Iniguez', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    91,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Hotel Reward token ',  -- title
    'Still many people not know how to use crypto currency and they are very scare or think it is very high tech',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'dazgpr5CJ9fsTEUNyIRrhsZ/F5M3o0D0MjHOnR/4uqw=', -- Public Payment Key
    '5000', -- funds
    'http://ideascale.com/t/UM5UZBdrV', -- url
    '', -- files_url
    113, -- impact_score
    '{"solution": "Join learning about crypto currency with hotel stay and reward them in Token and token can be use to pay for hotel and purchase product from"}', -- extra
    'Greencountryinn', -- proposer name
    '', -- proposer contact
    'https://greencountryinn.co', -- proposer URL
    'Ever since come from INDIA in 1998 working in hotel started as housekeeper and since 2002 Running and owning a small motel in oklahoma', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    92,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Artano - A Cardano NFT Marketplace',  -- title
    'The NFT market is not accessible to all. High fees and selectivity are barriers to entry for many artists as well as potential collectors.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'iRRsn4fr8kj6GBb0WZHIp1YUFJA6OeURvwFiIcF6LsU=', -- Public Payment Key
    '30000', -- funds
    'http://ideascale.com/t/UM5UZBdq5', -- url
    '', -- files_url
    345, -- impact_score
    '{"solution": "Building an NFT art marketplace upon Cardano where exclusivity & high fees are not a barrier to entry to artists and collectors."}', -- extra
    'Matt (Artano)', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Relevant experience

Matt (Software Developer/Artist)

Marija (Financial Consultant)

Ron (PM)

Sandip (Software Developer)', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    93,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=3), -- objective
    'PANGEA - Society Design Sandbox',  -- title
    'How can we apply gamification for rapidly spreading Cardano''s capabilities in a joyful and easy way to a large number of new participants ?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=3), -- category - VITSS Compat ONLY
    'EnQxAoAEr5YgXObkcvQNvPzcLpqbqIwJdsheIQ2vzY0=', -- Public Payment Key
    '250000', -- funds
    'http://ideascale.com/t/UM5UZBdqr', -- url
    '', -- files_url
    500, -- impact_score
    '{"brief": "**++PANGEA - The basic story plot:++**  \nIn order to allow fast progress and in order to avoid any issues with existing authorities and governments, the story-plot establishes a playful science fiction world: PANGEA, very similar to our home-planet. Same dimensions of land and oceans - but all continents metaphorically merged to one continent called PANGEA. Basic theme is the gamified foundation of stable new settlements on this Earth-like planet. The settlers would still be in well-integrated contact with home-planet Earth, but sufficiently far away to be forced doing independent experimenting.\n\n++**FUND 5 CHALLENGE: Focus on conception, MVP-developments & piloting.**++ This would set the frame for a larger number of proposers and partners, contributing to the necessary developments. As an outcome, we would establish the core teams and would jointly implement and pilot the basic technologies and concepts.\n\n++**Second Phase: Scaling (Fund X and/or based on external co-funding)**++  \nOnce proven in careful piloting, we would start the scaling phase. This is all about signing up ''players'' and metaphorically establishing and registering the highly decentralized habitats on PANGEA. The first focus would be put on (playfully) securing basic survival with setting up energy, water, hygienic, food-supply and basic agricultural production, IT infrastructure and controlling of other vital parameters (in reality: starting to apply our concepts by making our real earthly homes as decentralized as possible, including utilities supply, growing some food, establish excellent IT infrastructure etc. - a source of a careful and never-ending merchandizing income stream).\n\n++**Next phase: Establishing and growing the virtual PANGEA society**++  \nThen, the challenge would continue with grass-root-refinement of a new form of highly decentralized PANGEAN society composed of independent individuals in small ''settlements'' (should of course be a playful metaphor for our own homes and apartments). Following the growing number of settlers and settlements, the story-plot would lead to establishing the more and more complex governance structures and the PANGEA constitution based on the initial constitution draft and the CARDANO technology framework to support this process. After this, the game would move on and on, adding many new participants. Initially we would still follow our story-plot. Then, step-by-step, the framework could enable regional and individual sub-plots and potentially open a self-governed and non-plotted future.\n\n**++Conclusions++:** **Key of all this is to rapidly scale-up Cardano among a very new user-group. We can learn the ABC of decentralized and algorithms-supported governance. This way we contribute to building a better world by gaining real practical experience based on fun, openness and compassion.**", "importance": "Cardano offers cool technologies for building a better world. Choosing a Reality-Game format allows us to implement, test and scale this NOW", "goal": "A large, diverse and inclusive community designing, applying and testing Cardano technology and governance along a fascinating story plot.", "metrics": "The following could be established (and measured) as outcome of the Fund 5 funding:\n\n*   **Storyline & Scalability**: Selected teams to co-develop and refine the initial PANGEA story-plot. Result: Draft in place, with basic concept for global scaling and years of exciting action.\n*   **Core-Teams**: Focus on setting up interdisciplinary, diverse and inclusive core-teams\n*   **Risk Management:** Basic (non-controversial) legal frameworks investigated and in place for the major global regions, where we expect future game-participants.\n*   **Collaboration & Co-funding**: Establish concept for public co-funding (e.g. EU, WEF, global universities & school collaboration)\n*   **Long-term income-stream:** Establish concept for merchandizing, fees for piloting & sandbox-testing of decentralization-relevant technologies, concepts & tools\n*   **Core-Applications:** Define existing Cardano dApps to be applied or new ones to be developed (ID- and property-registration, governance and voting, basic concepts for gamified financial systems, social security and insurance contracts, smart contracts-based business platforms and new social media-systems etc.)\n*   **PANGEA Web-portal:** Establish highly attractive entry-point for the interactions between participants: Basic concept & piloting in place.\n*   **PANGEA Constitution:** Draft & process prepared for initial refinement, voting & piloting\n*   **Other metrics** according to plot and later roll-out progress: Number of registered participants = ''settlers'' (ID), Registered settlements & assets e.g. in a playful property register."}', -- extra
    'heinz.gassner', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    94,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Tournaments for Tennis Players',  -- title
    'Non-professional tennis players want to compete too. There are centralized ledgers/leagues. But we depend on 3rd party to organize/govern.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'B2MVVU6sxfsok28QPQAiA5tHAtuqSxVi1byf4OMeNdk=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBdqQ', -- url
    '', -- files_url
    117, -- impact_score
    '{"solution": "Create distributed ledger allowing players self-organize matches, keep scoreboards/ratings, and set own rules."}', -- extra
    'Anton', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I play tennis 2-3 times a wee.  
I''m a programmer myself and I run a small dev team for 15 years by now.  
We never did any dapps before though.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    95,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=3), -- objective
    'Governance Symposium',  -- title
    'What building blocks does the Cardano community need to to achieve truly decentralized governance?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=3), -- category - VITSS Compat ONLY
    'Vdxz3P3GLiNXYBkxWCQFjy2PJEXq7jow2nwiFnzls00=', -- Public Payment Key
    '250000', -- funds
    'http://ideascale.com/t/UM5UZBdqH', -- url
    '', -- files_url
    500, -- impact_score
    '{"brief": "The premise of this challenge is based on the idea that the Cardano Community must move forward (in a structured way) towards increasing self-governance. This process should be inclusive of both ''The Big 3'' (Cardano Foundation, Emurgo and Input Output Global) and the community at large. We should celebrate both submissions and involvement from these organisations.\n\nI am suggesting this will be the first of 3 rounds of challenges:  \n\n1.  **Governance Symposium** to canvas for ideas and building blocks.\n2.  **Constitutional Convention**. Once we have seen the research and ideas from the community and professionals we would invite submissions which propose a ''complete solution'' that puts these building blocks together for the community to vote on. There would be little funding (maybe none?) for this challenge, but submitters would be invited to propose projected implementation costs and stages. We would have a single ''winner'' from the community vote.\n3.  **Implementation**. The exact details of this would flow from the outcome of round 2, but with the community now understanding the desired ''end goal'' we can break out the different pieces needed to move forward and invite multiple parties to propose how they would build/implement the specifics.\n\nThe exact nature of the proposed 3 rounds might change as the community responds, but awareness of this should help frame the mindset for responders to this (the first) round. Having said that... these building blocks will still be independently useful to the community''s self-governance journey, even if we change our approach in a more fundamental way.", "importance": "Many layers to this problem, including election practices and structures; funding models; legal entities and agency; implementation bodies.", "goal": "Proposers would submit things like research papers; positions of legal advice; or open source code to contribute to ongoing dialogue.", "metrics": "Submissions should be concrete and detailed, certainly more than just simple advice or ideas.  \nSome ideas on the sorts of submissions that would help the community move forward:\n\n*   An open source library of code that can add one or more capabilities to an on-chain DAO, such as elected positions, multi-sig voting, periodic funding release.\n*   Protocol level research into how Treasury funding should be released through wallets that have key holders.\n*   Academic research into different governance systems that people can be elected into along with various pros/cons on what sort of ''balance of power'' mechanisms they create.\n*   Lawyers from specific jurisdictions (Wyoming? Switzerland?) that would venture some legal opinions on how to structure a not-for-profit foundation with a constitution/charter that requires them to ensure the existence, integrity and election of a DAO that could function as its ''Board of Directors''\u2026 or other such scenarios that integrate on-chain models with the real world legal systems.\n*   Research into how open source foundations (particularly anything blockchain related) protect themselves whilst still moving forward into the future\u2026 e.g. copyright, trademarks, law suits, funding models.  \n    There will be many more avenues of interest to pursue\u2026 these are just to give people ideas about the types of ''deliverables'' we might expect to see.\n\nFor Community Advisors, I am proposing the following scoring categories for this challenge:\n\n*   **Impact.** The proposal should be tightly focused on governance problems that are of relevance for the Cardano community and/or the Catalyst project. Avoid proposals that are overly vague or try to solve only ''real world'' election problems without grounding them in a Cardano context.\n*   **Reusability**. The proposal should not be putting forward a solution that cannot be picked up by other teams in later rounds as part of a larger governance framework. e.g. Technical proposals that are not open source, or too specific (like focusing on running a small club).\n*   **Auditability**. The proposer has provided some verifiable details on why they are a good person to undertake this work, along with some mechanisms for the Voting Committee to perhaps release funding in a staged way. e.g. an academic paper might consider exposing a rough outline and/or early drafts at a set of date milestones to the Voting Committee."}', -- extra
    'Greg Pendlebury', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    96,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'CHESS - 600 Million People',  -- title
    'Chess APP incentivising Grandmasters to play lower ranked players + bespoke AI puzzles to improve each INDIVIDUAL player''s game',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'F3RfmkF7HMVZIzhPGw77icDgP/ARDn3PDzlcRSbLVn0=', -- Public Payment Key
    '100000', -- funds
    'http://ideascale.com/t/UM5UZBdp9', -- url
    '', -- files_url
    131, -- impact_score
    '{"solution": "Utilising blockchain to facilitate payments in cryptocurrency and also in rating between players. Play for rating points or crypto or both."}', -- extra
    'PlanetStake', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I like to play chess and would like an arena where great players are incentivized to come and play those still learning to master the game.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    97,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Business Governance Application',  -- title
    'Ineffective business governance leads to disengaged employees which costs the U.S. $485-605 billion per year \[1\].',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    '+OhACNqwkgnvsmOEOSBK8bP0Uc2fKBIKmwm+ypH75I0=', -- Public Payment Key
    '97849', -- funds
    'http://ideascale.com/t/UM5UZBdp1', -- url
    '', -- files_url
    144, -- impact_score
    '{"solution": "Increase engagement by automate project assignment, based on employee strengths, current workload, and management/team voting system."}', -- extra
    'bchristian14', -- proposer name
    '', -- proposer contact
    'https://www.linkedin.com/in/william-christian-05a21b74/', -- proposer URL
    'Over the last 2 years I have increased the productivity of teams I have led by 187% by programming tools for effective governance.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    98,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'e-invoicing standard',  -- title
    'E-invoicing project are hard to put in place. They need tx, routing and rules that must be established in a project specification.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'trPJHwbLodE1ycam28cQq5L+9q0F/q/Vw5S62CDex6s=', -- Public Payment Key
    '22500', -- funds
    'http://ideascale.com/t/UM5UZBdpv', -- url
    '', -- files_url
    263, -- impact_score
    '{"solution": "e-invoice as NFT with metadata and smart contract (self-invoicing, periodic invoices,...)"}', -- extra
    'Jean', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I''ve built smart contract before, I''m a senior developer, I use to have a company selling to other companies through ERP/EDI prior', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    99,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Decentralised Funding Report',  -- title
    '**See PDF attached**

Decentralised funding is a highly complicated system/process and we still need to figure out how to make it all work.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    'EhqRX3AlrBe9TM22kJeUu1hxA1ojc9cZrcj1K8j0AkM=', -- Public Payment Key
    '6109', -- funds
    'http://ideascale.com/t/UM5UZBdpY', -- url
    '', -- files_url
    200, -- impact_score
    '{"solution": "**See PDF attached**\n\nWe were involved in a similar project before which worked very well and **++we can write out a report to learn from it.++**"}', -- extra
    'icoresearchgroup', -- proposer name
    '', -- proposer contact
    'https://icoresearchgroup.com/', -- proposer URL
    '**See PDF attached**

*   Experience with decentralised funding systems
*   Been in the Finance space for over a decade
*   Degree in Economics', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    100,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'CardanoSharp - .NET Library',  -- title
    'There is no common libraries to help .NET Developers build on Cardano.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    'WI2zHNTltLf//9QInaebeSXvpvJY5axISIX3dY/Cqbw=', -- Public Payment Key
    '19520', -- funds
    'http://ideascale.com/t/UM5UZBdpU', -- url
    '', -- files_url
    465, -- impact_score
    '{"solution": "Build a collection of libraries that will help facilitate .NET development for Cardano."}', -- extra
    'Kyle | LIFT', -- proposer name
    '', -- proposer contact
    'https://github.com/CardanoSharp', -- proposer URL
    '.NET Developer with over 10 years. Currently managing dozens of microservices over multiple environments using latest .NET Technologies', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    101,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=3), -- objective
    'Decentralized Sources of Truth',  -- title
    'How can we ensure that future voters in the Cardano ecosystem will vote in a way that benefits the health of the ecosystem and themselves?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=3), -- category - VITSS Compat ONLY
    '87h2+wRkikOpBt7X6ndHv/XefGKCXFgXtdN1fvH5zzs=', -- Public Payment Key
    '250000', -- funds
    'http://ideascale.com/t/UM5UZBdo7', -- url
    '', -- files_url
    500, -- impact_score
    '{"brief": "In the past decade, we have seen that the opinion of a collective group of people can be manipulated, literally steered, toward the desired conclusion of some entity through social engineering. The main driver of this is the sharing of misinformation on social media. Social media platforms profit by keeping eyes on screens, which incentivizes the platform to validate a user''s confirmation bias as they are shown information they WANT to believe, but may not necessarily be true.\n\n  \nThe objective of this challenge is NOT to come up with a solution for the sharing of misinformation on social media or to create a new decentralized social media platform. That is a big problem with big questions and seems to be outside the current scope of Project Catalyst. Instead, it is more prudent to focus on practical solutions that can be implemented in the short-term and can have an immediate impact on the community.\n\n  \nIn this spirit, perhaps efforts towards helping individuals recognize misinformation, or giving them methods to validate information in a decentralized and trusted way is more worth our time and money. The Cardano subreddit is one of the most active cryptocurrency forums as measured by both comments and posts per user per day\\[1\\] . This recent influx of new users asking questions about the Cardano protocol and participating in the discussion on social media channels demonstrates that solutions for conveying trustworthy information for all levels of education is in high demand. It is important that methods of preventing social engineering be developed now, while the Cardano protocol is still young and in its formative years.\n\n  \n\"*Elevator Pitch*\"  \n**Therefore, the objective of this proposal IS to focus on the development of a trustful method to help community members of all levels of education find, share, and verify information about Cardano that each individual can know is true. The result of this future Fund5 challenge will bring about solutions to battle misinformation designed to harm the network through education.**\n\nMajor decisions about Cardano will be made by the community via its voting mechanisms. Cardano is a social experiment just as much as it is a technical one, which is why it''s imperative to have mechanisms in place to help people make the best decisions possible for the health and future of the network. It will be easy to make \"naive\" conclusions about what may benefit the long-term health and growth of the network because:\n\n  \n\n  \n1) voters may not be familiar with the game-theory that was initially used to develop the incentive mechanisms that were designed to facilitate trends that would lead towards long-term health of the network. In turn, voters may make decisions naively, thinking a voting proposal is in the interest of the network but, in fact, is not.  \n2) voters tend to make decisions myopically by focusing on short-term, personal gain as opposed to long-term network benefits.\n\n  \n\n  \n\nA social engineering attack designed to leverage these two scenarios could be the first phase, an opening of the door, to a greater attack designed to damage the network or its long-term health. So, it seems reasonable that there should be a check on these two scenarios to proactively defend against threats. The systems being developed to address the two above issues, along with their drawbacks are discussed below.\n\n  \n\n  \nIssue 1:  \nVoters may not be familiar with the game-theory that was initially used to develop incentive mechanisms that were designed to facilitate trends that would lead towards long-term health of the network. In turn, voters may make decisions naively, thinking a voting proposal is in the interest of the network but, in fact, is not.  \n  \n\nProposed Solution:  \n\\- A \"Liquid Democracy\" system that will allow voters to delegate their vote to an \"Expert\" in a manner they see fit when they feel they are not in a place to make a wise decision  \n  \n\nPotential Drawbacks:  \nCurrently, for the greater population of voters, the primary source of information to make a voting decision is whatever is provided by the Catalyst Voting mobile app. The information provided in the app by each proposer goes through multiple rounds of revision by community advisors, which provides rigor. As great as that process is, there will ultimately be proposals on decisions that will require knowledge that is outside the scope of an average individual''s knowledge or abilities. The \"Liquid Democracy\" model will allow a voter to delegate their vote in this scenario.\n\n  \nHowever, the problem still stands; If an individual does independently decide that their vote needs to be delegated, an informed and educated decision still needs to be made. Whom is the individual to trust when making the decision to delegate their vote? Can the expert be trusted? The expert has an incentive to gather voting power and may say whatever is necessary to get it. The expert could put false information out there, or partner with an entity that will propagate false information in the interest of the \"expert\". The expert''s motivation could be to intentionally do harm to the network or, even worse, may think their intentions are good, but have, in-fact, succumbed to misinformation.  \n  \n\n  \n\nIssue 2:  \nvoters tend to make decisions myopically by focusing on short-term, personal benefits as opposed to long-term network benefits  \n  \n\nProposed Solution:  \n\\- Financial incentives to participate in the voting process  \n\nPotential Drawbacks:  \nVoters, as well as Experts, will be financially incentivized to participate in the voting process. Research shows that this will boost turnout \\[2\\]. However, this does not incentivize educated voting decisions. Voters could submit ballots by just checking off one choice, perhaps whatever is listed at the top, putting in the least effort possible to receive a reward. Another scenario; voters could checkoff all choices in the misguided thinking that they will receive more rewards with more ballots cast. Therefore, the financial incentive is not enough of a check to ensure good decision making. Greater checks can be put in place if the incentives for decision making in the short-term align with desired outcomes to protect the long-term health of the network. The preceding examples are of the extreme case; there is perhaps no way to incentivize those individuals to participate in a valuable way. But that is exactly why this challenge is so important.  \n  \n\n  \n\nThis project is feasible because:\n\n*   There are similar constructs out there that already exist for sharing content. Another''s business model could be used as a starting point\n\n  \nThis project is auditable because:  \n\n*   The following requirements can be used to facilitate auditability:  \n    Within 60 days of the close of voting for Fund5, winners of funds will publish a whitepaper outlining and describing:\n*   How the funds will be budgeted\n*   How the below would be achieved and on what timeframes:\n*   Creation of a dApp that:\n*   Serves as a repository and curation platform for content about Cardano. Hosted content should be of multiple modalities (text, video, audio, etc.)\n*   Allows users to verify accuracy of information via links to primary sources\n*   Incentivizes content creators to create or submit content that:\n*   is engaging and factual. It can be verified through primary sources\n*   is diversified so that it meets the needs of a wide range of educational levels, cognitive abilities, and cultures\n*   Incentivizes content consumers to:\n*   verify content\n*   Promotes education as a method to help make an informed decision\n*   Helps promote social incentives for voting in a manner that benefits the ecosystem to help deter the threat of future social engineering intended to harm the network\n\n  \nThis project is impactful because:  \n\n*   There is a high demand for knowledge and education about Cardano and it will only continue to grow. This project proactively reduces the potential that misconceptions and misinformation will propagate\n\n  \n\n  \n\nCONCLUSION\n\nI put this Community Choice Challenge out to the community to try to devise clever ways to possibly create extrinsic motivation for even the most extreme cases. Perhaps the solution creates a social incentive. Maybe a reputation system is developed (similar to stack exchange) to incentivize people to create and communicate accurate, high quality content. As opposed to stack exchange''s votes, maybe real ADA could be spent to validate accurate, high quality content. Real ADA could be earned for the creation of content. These are just suggestions and not part of the requirements for auditability of the challenge. Again, this challenge is not intended to create a decentralized social media platform. Think more towards the direction of Wikipedia, not Twitter. I don''t see my role as coming up with the solution, we should fund this challenge so we can discover what the clever Cardano community can dream up!  \n\n**Sources:**\n\n*   \\[1\\] https://www.reddit.com/r/cardano/comments/l73uhn/cardano_doesnt_have_the_largest_community_yet_but/\n*   \\[2\\] https://www.washingtonpost.com/opinions/2019/04/04/how-do-we-get-people-vote-lets-try-financial-incentives/\n\n**Additional Resources:**  \n\n*   https://scholar.harvard.edu/files/glaeser/files/democracy_final_jeg_1.pdf\n*   https://www.psychologytoday.com/us/blog/cutting-edge-leadership/201712/why-do-people-vote-against-their-best-interests\n*   https://www.theatlantic.com/education/archive/2018/10/civics-education-helps-form-young-voters-and-activists/572299/", "importance": "Voters do not always vote in their own best interest. The monetary incentive scheme being developed is a great start, but it is not enough.", "goal": "Voters would have a de facto dApp/website they could visit to conduct research. It would present information in varying modalities, for all", "metrics": "*   Creation of a dApp that:\n\n*   Serves as a repository and curation platform for content about Cardano. Hosted content should be of multiple modalities (text, video, audio, etc.)\n*   Allows users to verify accuracy of information via links to primary sources\n\n*   Incentivizes content creators to create or submit content that:\n\n*   is engaging and factual. It can be verified through primary sources\n*   is diversified so that it meets the needs of a wide range of educational levels, cognitive abilities, and cultures\n\n*   Incentivizes content consumers to:\n\n*   verify content\n\n*   Promotes education as a method to help make an informed decision\n*   Helps promote social incentives for voting in a manner that benefits the ecosystem to help deter the threat of future social engineering intended to harm the network"}', -- extra
    'talking.martlet', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    102,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Developer & SPO Tools [CNTools]',  -- title
    'Developers need easy to use tools to query and analyze the blockchain as well as integration opportunities for adoption.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    'ynGUdzxu/QZibUBiuzhd/O55bvYlq85x8MhYfVZDOVk=', -- Public Payment Key
    '14500', -- funds
    'http://ideascale.com/t/UM5UZBdo6', -- url
    '', -- files_url
    400, -- impact_score
    '{"solution": "Provide integrations in CNTools to easily query chain data and integrate third-party developed applications."}', -- extra
    'ola.ahlman', -- proposer name
    '', -- proposer contact
    'https://github.com/cardano-community/guild-operators', -- proposer URL
    'Ola Ahlman - SPO since 2019 (ticker: AHL)  
Background in air traffic control systems and deploying mission-critical IT systems.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    103,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Cardano TV - Sports LED Advertising',  -- title
    'TV Sports advertising is eye catching, dynamic and creative, Cardano will be the 1st to reach 1000s of Developers & Entrepreneurs eyes - WOW',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    'ZL2AqwdviebmkrXS6qXp2o+9pCOXGH6E4qX5UfUgIrY=', -- Public Payment Key
    '16000', -- funds
    'http://ideascale.com/t/UM5UZBdoq', -- url
    '', -- files_url
    157, -- impact_score
    '{"solution": "TV Advertising puts the Cardano brand in front of people and offers credibility. Fully engaged viewers can be uniquely attracted to Catalyst"}', -- extra
    'Rob Greig', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Over 8 years'' experience building LED TV adverts with 100s International Brands, at 100s stadiums and 10+ sports Worldwide having 1m+ views', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    104,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'TRUST: Debunk misinformation DApp',  -- title
    'Quality information scarce. Misinformation is crippling our ability to make (the right) choices and is taking power from the people.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'el2+INLIWBbmNIplQVxKkB3ZBZDkcHIMpFFcYlUv5Kk=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBdoh', -- url
    '', -- files_url
    142, -- impact_score
    '{"solution": "TRUST: dapp that guarantees information closest to truth. Only allowing raw data and inscentified, gamified, falsification for the masses."}', -- extra
    'lars', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'As for my track record in the space: NADA:) This is just a proposal and if it is a good idea the right people will appear.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    105,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'TCG fractional-share marketplace',  -- title
    'With highly sought after chase cards reaching new highs, many graded card collectors are priced out from ownership.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'PMUH8TgWGNNCaqcwWXANkorwymSKqLYAub3MCRD3CXA=', -- Public Payment Key
    '11000', -- funds
    'http://ideascale.com/t/UM5UZBdoX', -- url
    '', -- files_url
    143, -- impact_score
    '{"solution": "A marketplace where users can purchase fractional ownership of real graded cards (insured and in a vault)."}', -- extra
    'Giovanni De La Rosa', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Some programming; business marketing', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    106,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Cardano IntelliJ IDEA Plugin (MVP)',  -- title
    'Plugin to support development on Cardano inside IntelliJ IDE. One of the major blocker for mainstream developers is unfamiliar / no tooling.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    'yDWSeVnlQF+byDt0m6qttqeg7Xx3o4PzFToDPbrzji4=', -- Public Payment Key
    '15000', -- funds
    'http://ideascale.com/t/UM5UZBdoW', -- url
    '', -- files_url
    450, -- impact_score
    '{"solution": "Build an IntelliJ plugin with following features:  \n\n\\- Configure a node in IntelliJ\n\n\\- Account mgmt\n\n\\- UI for Transaction & native token mgmt"}', -- extra
    'Satya', -- proposer name
    '', -- proposer contact
    'https://www.bloxbean.com/', -- proposer URL
    'Experienced software architect and developer. Exp in Dev Tooling.

Prev work: IntelliJ Plugin for Algorand, Aion

Maintaining BloxBean pool', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    107,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Proof of Ownership-Land/Real Estate',  -- title
    'Many lack confirmation of ownership to parcels of land they occupy, or they become displaced and are not able to provide proof of ownership.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'FLHp1wn/3XVyo4QMtZyi5lk1mtEmHdYoX3hVCJR2geo=', -- Public Payment Key
    '25000', -- funds
    'http://ideascale.com/t/UM5UZBdoR', -- url
    '', -- files_url
    220, -- impact_score
    '{"solution": "Land plots are created using GPS coordinates for initial ownership, subsequent purchase/sales are made using ADA on a verifiable blockchain."}', -- extra
    'Aaron DeKosky', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I''m a lawyer who partnered with the co-founder of a cloud computing company.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    108,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=3), -- objective
    'The charity fund',  -- title
    'How can we help towards ending poverty and water scarcity, create peace in war torn areas, improve sanitation and hygiene in areas of need.',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=3), -- category - VITSS Compat ONLY
    'BrEu4gnq60uw3I93Lw1csFdBVVvLcvMOMkFQtHiYXkc=', -- Public Payment Key
    '200000', -- funds
    'http://ideascale.com/t/UM5UZBdn9', -- url
    '', -- files_url
    500, -- impact_score
    '{"brief": "How can we help towards ending poverty and water scarcity, create peace in war torn areas, improve sanitation and hygiene in areas of need.", "importance": "Hunger and malnutrition are in fact the number one risk to health worldwide \u2014 greater than AIDS, malaria and tuberculosis combined.", "goal": "Any improvement in any amount of lives is a success.", "metrics": "Number of lives saved or improved"}', -- extra
    'Lucky', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    109,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Crowdfunding/DAO opensource pharma ',  -- title
    'Pharma funding is heavily centralised, many diseases lack treatments. Scientists trying to discover new treatments struggle to get funding.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'ZDy7hADobn+BwNcJY/HDa2O3KokZiYAX9F5pVZVjW+Y=', -- Public Payment Key
    '60000', -- funds
    'http://ideascale.com/t/UM5UZBdm6', -- url
    '', -- files_url
    240, -- impact_score
    '{"solution": "Open source, decentralised drug discovery (the Github for drug discovery) with crowdfunding (via augmented bonding curves) & DAO grants."}', -- extra
    'Andre.Chagwedera', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '9+ years in health and life sciences. Selected to be on a startup accelerator based in London. Have scientific advisors with decades of exp', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    110,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=3), -- objective
    'Basic Marketing Campaign ',  -- title
    'The existence of Bitcoin is just being discovered by the general public, it is our duty to make ourselves known at such a crucial time.',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=3), -- category - VITSS Compat ONLY
    'ikzzV0MmofWyKMFK2TSCOnIf5OonhqWHYd7VYftJFAw=', -- Public Payment Key
    '2000', -- funds
    'http://ideascale.com/t/UM5UZBdmu', -- url
    '', -- files_url
    500, -- impact_score
    '{"brief": "I''d like to invest some money into social media advertisements to kick off with. If this is successful we will expand to more high yield platforms. My day-to-day life is to work as a hospital doctor in the NHS, but I have led similar projects in my university years and I am surrounded by people that can help to give it a solid attempt.", "importance": "Cryptocurrencies carry with them a lot of stigma and fear. We as a community need to address that through marketing.", "goal": "Recognition by the mainstream media", "metrics": "The increasing number of members on our platforms (Reddit, Discord, Telegram, Twitter, etc)\n\nIncreasing the number of google searches"}', -- extra
    'Qthulhu', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    111,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'ABCD',  -- title
    'Africa has brilliant minds that are full of ideas and energy but who lack access to the infrastructure and funds to realize their dreams.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    '/lXtwC2KiBvgByKLVRNOT43tyvfMrc1tsVGuME7ojdo=', -- Public Payment Key
    '13800', -- funds
    'http://ideascale.com/t/UM5UZBdmq', -- url
    '', -- files_url
    403, -- impact_score
    '{"solution": "ABCD will create opportunities for Africans providing blockchain solutions to the world utilizing the pool of talent working from Africa."}', -- extra
    'Joshua Akpan', -- proposer name
    '', -- proposer contact
    'https://poapool.com/projects', -- proposer URL
    'ABCD founders have been long involved in the development of initiatives related to the tech industry and blockchain in Nigeria and Africa.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    112,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=3), -- objective
    'Cardano Television Commercial',  -- title
    'How do we tell the world about the new world financial operating system in the most impactful manner?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=3), -- category - VITSS Compat ONLY
    'JCqS1lu3pYsF0rS238/iszX0rkqf0PHWA3M0Jxb9HYc=', -- Public Payment Key
    '100000', -- funds
    'http://ideascale.com/t/UM5UZBdmd', -- url
    '', -- files_url
    500, -- impact_score
    '{"brief": "Cardano needs a defining moment, something similar to the \"1984\" Super Bowl commercial announcing the release of the Apple Macintosh. The \"Trust Cardano\" campaign will coincide with the release of Goguen.\n\nThe BIG problem is that trust, in our institutions, in our media, in each other, has been broken.\n\nThe issue is TRUST. We tell people they can trust Cardano. We give people hope.\n\nI have written the script and am looking for someone to produce it. The funding would go toward the making of the 30 second commercial. As this may be a substantial cost, I propose that the cost of the airing of the commercial would be voted upon separately in fund 6.\n\nIn addition, a powerful, Oscar-grade commercial entirely produced by the community and funded by Catalyst would be a great venue through which to showcase the Catalyst project itself. Essentially we can market Cardano (via the commercial itself) and Catalyst (talking about the commercial via crypto podcasts/You Tubers), giving us more marketing bang for our buck.", "importance": "If Cardano is going to change the world, the world needs to know of Cardano. Marketing to the general public has thus far been non-existent.", "goal": "Mass adoption of the Cardano protocol by the general public.", "metrics": "Cardano search phrase metrics.\n\nIncrease in wallet creation.\n\nMore ADA bought and staked."}', -- extra
    'Roy Dopson', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    113,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'In-Store Currency (Shopify)',  -- title
    'Stores manage different centralised value mechanisms including credit, loyalty & gift cards which adds complexity, cost & limits innovation.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'ItkleMeN4vVAxCoRK8NCZwiODi7GfDw+xhxHhmk35+4=', -- Public Payment Key
    '100000', -- funds
    'http://ideascale.com/t/UM5UZBdmY', -- url
    '', -- files_url
    281, -- impact_score
    '{"solution": "Unify the use of in-store currency that can be issued by the store and/or transferred by the customer to be redeemed through checkout."}', -- extra
    'dolythgoe', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Software and UX team of 4 - operate a Shopify discount app and Cardano stake pool. Worked in ecommerce for large brand names.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    114,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Decentralized Accounting <- IFRS',  -- title
    'Companies are using balance sheets to keep their books 📒.

They should be able to do this on the Cardano Blockchain.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    '0706nA6utGkqb/32oCA6+wKxOAPxRLFSc2xCLGw35Es=', -- Public Payment Key
    '4000', -- funds
    'http://ideascale.com/t/UM5UZBdmV', -- url
    '', -- files_url
    396, -- impact_score
    '{"solution": "I want to implement a blank balance sheet in Plutus."}', -- extra
    'quirin.schlegel', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I am studying information systems management at the TU Berlin, where I learned to code in Haskell and how to do Accounting.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    115,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=3), -- objective
    'Cardano Faucet Challenge',  -- title
    'Can you create a faucet to encourage adoption or usage of Cardano over the next 6 months?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=3), -- category - VITSS Compat ONLY
    'V/2fId0Rr84QgX0G9UomXSos1z+HbyXr/8bABdgzh2w=', -- Public Payment Key
    '100000', -- funds
    'http://ideascale.com/t/UM5UZBdl9', -- url
    '', -- files_url
    500, -- impact_score
    '{"brief": "Create some interesting, fun, secure Faucets that promote Cardano  \nExamples of other cryptocurrencies Faucets  \nsatoshiquiz -Bitcoin  \nBlack MonKeys - Banano  \nAdafrogs bubble pop game - Cardano  \nSocial media tipbots  \nCoinbase Earn - multiple currencies\n\n  \n**++Addressing the Assessments++**\n\nFirstly if you assessed my proposal or even if you just read this far - Thanks!\n\n  \nExperience Worries - You don''t understand the challenge. I am not making the faucets, I haven''t proposed my idea disguised as a challenge. Address these to those who actually submit ideas to the challenge if/when it gets through to fund 5. I''ve pointed this question out as unneccessary (in a community fund proposal setting) to the catalyst team but understand it is early days, things will get reused that need to be asked of actual projects.\n\n  \nFunding Allocation - Of the $80k ($100k less the 20% in rewards to voters) I think it should be $79,999 to one proposer to build a faucet that gives away $1  \nseriously though its not up to me to set guidelines for the proposers but I imagine 4-5 winners asking for $16k-20k each with $1k-2K to build it and a $14k-19k as faucet funds if you really want to know my perfect outcome\n\n  \nWallet Adoption Metric - Yeah this could be hard to measure i was just chucking things out there that people may want to measure for success\n\n  \nCardano Doesn''t Need Faucets, it''s Great the Way it is - You''re probably right, but it may be fun for some people.  \nPeople Won''t Do Anything for Small Amounts of ADA from a Faucet - Tell that to the people who complained in the bitcoin forums to Charles about only getting paid 0.05btc for answering a question\n\n  \n\nI do wish we''d been able to discuss these points in the comments before the assessment stage but i understand there were so many proposals to read through, thanks again!", "importance": "Adoption & Fun\n\nMay lead to Dapps, on-chain use & promotion of information about Cardano", "goal": "Interest & adoption by the wider crypto community\n\nAn ongoing Faucet(s) that continues to draw people into Cardano\n\nFun for our community", "metrics": "Has the faucet been used? - have the reward funds been used up\n\nDid it increase user wallet adoption?\n\nDid it use on-chain features? E.g. metadata or Dapps\n\nDid it promote information about Cardano? Integration with current ADA infrastructure e.g. pooltool, adapools\n\nWas it secure? Privacy of participants, lack of bot attacks\n\nGrowth e.g. mentions across ADA social media\n\nWere the \"tokenomics\" correct? Did it provide too much or too little reward for participating?\n\nCost of creation vs prize fund amount\n\nBonus Metric: ease of use or \"Proof of Dad\" Get Charles to mention in one of his AMA''s that his Dad used your faucet"}', -- extra
    'Marshosaurus', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    116,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=3), -- objective
    'Have an open fund',  -- title
    'Have an open fund? Instead of multiple ''challenges'' with restricted $. Therefore allowing the voting to choose what''s important.',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=3), -- category - VITSS Compat ONLY
    '5+9Zz0ZDjfq82mIsWOOsoQR5IZTNFEkg3DQUWmWQ8LQ=', -- Public Payment Key
    '100000', -- funds
    'http://ideascale.com/t/UM5UZBdl1', -- url
    '', -- files_url
    500, -- impact_score
    '{"brief": "It''s not possible to know what $x is reasonable for each challenge without knowing how many solutions there will be for that challenge.  \nAn open fund allows the vote to choose what is most important or what proposals are the best without arbitrary restrictions of having to meet a challenge, or how much $ is allocated to the challenge.  \nAn open fund also means that the purpose of community advisors (whom costed $250 each in fund2) probably becomes moot. Which personally i think is an added benefit, because I believe they are an unnecessary cost that doesn''t help or influence voters anyway.", "importance": "It''s not possible to know what $x is reasonable for each challenge without knowing how many solutions there will be for that challenge.", "goal": "More proposals. Less confusion/subjectivity about what they need to meet.\n\nConversely, Failure looks like 4 proposals in ''community choice''.", "metrics": "How many proposals do you get."}', -- extra
    'Lucky', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    117,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=3), -- objective
    'Scale-UP Cardano''s DeFi Ecosystem',  -- title
    'How can we encourage DeFi teams to build/deploy open finance solutions on Cardano in the next 6 months?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=3), -- category - VITSS Compat ONLY
    'y1CcLlbktw4q1yS8nC41Fx0RxSZXsUJTL6HTePkVPUA=', -- Public Payment Key
    '250000', -- funds
    'http://ideascale.com/t/UM5UZBdlj', -- url
    '', -- files_url
    500, -- impact_score
    '{"brief": "Currently the vast majority of DeFi dapps and currency flows occur on the Ethereum blockchain, however due to congestion and increased transaction fees, the use of DeFi is increasingly elitized, leaving out the people who could most benefit from it. benefit from this, the unbanked.\n\n  \nTo make simple swap transactions between 2 tokens a person can spend more than 10 dollars. More complex transactions that require more space in blocks easily pass $ 100. While Ethereum 2.0 is not implemented, different solutions to the problem of congestion and high transactions fees are being developed, some involve the use of a second layer in Ethereum 1.0 (Zk rollups, Optimistic rollups, Plasma, etc.), others involve the creation of DeFi in other blockchains, like Cosmos and Polkadot, but this is not restricted to just these two, many blockchains are focusing on the development of DeFi.  \nThis image represents the ecosystem of DeFi-related projects at Polkadot.  \nhttps://pbs.twimg.com/media/Erz71y2XAAUO1Ek?format=jpg&name=large\n\n  \nThe current situation shows that there is a huge public that is unable to use DeFi due to high transaction fees and several projects are developing solutions for this. I believe that Cardano is an excellent option for this.  \nBut why Cardano?\n\n  \nThere is a concept that the DeFi community is well aware of, which is the composability of protocols / dapps. This allows several smart contracts of different dapps/protocols to interact with each other, this concept of composability is often referred to \"DeFi legos\", in analogy to the LEGO building toy. In this analogy, each dapp / protocol would be a lego and the combination of these can leverage DeFi, creating more options, services, liquidity, among other positive aspects, but it can also leverage risks from poorly programmed, unaudited contracts, excess leverage and risks in general.  \nImagine that the primordial dapps were at the base of a pyramid and more complex dapps were built above these. These articles are a good source of information on the subject:  \nhttps://medium.com/pov-crypto/ethereum-the-digital-finance-stack-4ba988c6c14b  \nhttps://medium.com/@roypritam1234/decentralized-finance-defi-stack-built-on-ethereum-feea8f93a47d\n\n  \nThe problem is that the money LEGOS composability brings systematic problems and this is exacerbated by the problems caused by the absence of code with formal verification and this makes Cardano an excellent option for critical applications, such as DeFi dApps.\n\n  \nIn addition to the advantage of having a functional language and portability with KEVM and other languages in the future, we still have MARLOWE, which should facilitate onboarding for people without technical programming knowledge and ATALA PRISM, which should help onboard people without identity documents / without credit score.\n\n  \nIf we want a rich and competitive environment for DeFi on Cardano we need to accelerate the development of these dapps and prioritize their development. With this challenge, we can encourage the creation of native DeFi projects on Cardano and promote Cardano as a scalability and security option for DeFi projects that are on Ethereum and are interested in migrating their project to Cardano. We can use the Catalyst referral program to attract more projects / proposals.\n\n  \nI will mention some project niches that I believe are important for creating a promising DeFi environment on Cardano, all the niches mentioned below have projects implemented (mainly on Ethereum blockchain) and can serve as inspiration for proposers who want to submit proposals on Fund5, if this challenge be chosen by the community.\n\n  \nFor more information on DeFi''s niche markets, I recommend accessing.  \nhttps://defiprime.com/\n\n  \n**I WANT TO CLARIFY THAT ALL THE EXAMPLES SHOWN BELOW ARE IDEAS OF WHAT CAN BE CREATED BY PROPOSALS ONLY IN FUND5, IF THIS CHALLENGE IS CHOSEN BY THE CARDANO COMMUNITY.**\n\n*   **ORACLES AND PREDICTION MARKETS** They allow the inclusion of \"real world\" data in the blockchain, for example, prices of digital assets and unique events, such as the result of a game, weather, elections, among others.  \n    https://chain.link/  \n    https://augur.net/  \n    https://tellor.io/  \n    https://omen.eth.link/#/liquidity  \n    https://explorer.ergoplatform.com/en/oracle-pools-list\n*   **DECENTRALIZED EXCHANGES (DEXs)** Within DEXs, different DEX models can be created, the most common of which are ++Order books DEXs++ and ++AMM DEXs.++\n*   **AMM (Automated Market Maker)** They enable a more efficient model of decentralized exchange, where sellers and buyers are available to execute a trade without the need for a match order. This model uses liquidity pools to create constant liquidity, essential for tokens with little trade volume.  \n    https://uniswap.org/  \n    https://balancer.finance/  \n    https://app.bancor.network/\n*   **ORDER BOOKS -** They present the same concept as traditional exchanges, where buyer and seller need their offers to have a price match for the trade to be executed, however with the use of blockchains this can be done in a decentralized way.  \n    https://loopring.org/\n*   **DEX AGGREGATORS** Allow users to get price arbitration between different DEXs in an automated and intuitive way.  \n    https://1inch.exchange/  \n    https://paraswap.io/\n*   **MONEY MARKETS & LENDING PROTOCOLS** They allow users to make loans without gatekeepers as intermediaries, only through smart contracts and the economic model of a dapp.  \n    https://www.liqwid.finance/  \n    https://compound.finance/  \n    https://aave.com/\n*   **FLASH LOANS** Allow users to use liquidity poools to borrow from a pool without providing collateral, contact to pay for the transaction in the same block + fee (the profit is left to the arbitrator). This allows atomic arbitration between different dapps by anyone who has technical know-how, but does not have the capital to perform the arbitration.  \n    https://aave.com/\n*   **LIQUIDATION POOLS** They allow users to pool their funds in a pool to liquidate users who don''t have enough collateral.  \n    https://atomica.org/\n*   **DERIVATIVES & OPTIONS** In traditional finance, a derivative is a contract that derives its value from the performance of an underlying entity. This underlying entity can be an asset, index, or interest rate, and is often simply called the underlying.\" Derivatives on DeFi allow this to be done transparently and with accountability, solving problems that exist in the current financial market, problems that have caused rehypotecation of papers (IOUs) in cases like Dole Foods. Algorithmic stable coins can also be understood as a product resulting from a dapp of that niche, since they can be seen as dollar derivatives, like Maker DAO stablecoi, DAI an the new model created by EMURGO, AgeUSD.  \n    https://makerdao.com/en/  \n    https://www.synthetix.io/  \n    https://opyn.co/#/\n*   **INSURANCE** It allows users to be able to cover possible failures and losses associated with a DeFi protocol.  \n    https://nexusmutual.io/\n*   **MARKETPLACES FOR DIGITAL ASSETS** Allow users to trade their digital assets in a decentralized way, the focus here are NFTs, as these are not possible to be traded with conventional DEX models.  \n    https://opensea.io/  \n    https://wax.atomichub.io/\n*   **ASSET MANAGEMENT TOOLS** They allow a dapp to manage his digital assets, through a single interface, automation in the execution of smart contracts and to execute contracts of other dapps within a single platform.  \n    https://yearn.finance/  \n    https://zapper.fi  \n    https://www.tokensets.com/  \n    https://mycrypto.com/  \n    https://defisaver.com/  \n    https://gelato.network/\n*   **ANALYTICIS & CONTENT AGGREGATORS** Analytics is the discovery, interpretation, and communication of meaningful patterns in DeFi protocols data and the process of applying those patterns towards effective decision making. Portals/websites that monitor the DeFi ecosystem, offer data, metrics and information material for projects related to the DeFi ecosystem. This type of project is fundamental for the introduction of new users to the DeFi ecosystem and the information of all people involved in DeFi.  \n    https://www.duneanalytics.com/  \n    https://defipulse.com/  \n    https://pools.fyi/#/pt/  \n    https://dappradar.com/  \n    https://www.dapp.com/", "importance": "It''s important b/c Defi dApps provide trustless on-demand access to financial services for global users w/ Total Value Locked ~$25billion", "goal": "A competitive environment for DeFi on Cardano with an emphasis on cross-collaboration & driving composability across protocols.", "metrics": "How many DeFi protocols were launched on Cardano mainnet within 6 months from this DeFi focused community challenge?\n\nHow many daily active users are the DeFi protocols funded in this cohort attracting?\n\nHow much Total Value Locked (TVL) are the funded DeFi protocols able to capture 1 month post launch on Cardano mainnet? (measure again at 3 months, 6 months and 1 year post-implementation to glean complete insights on proposal challenge effectiveness)\n\nHow many DeFi developers did this community focused challenge bring into the Cardano ecosystem?\n\nHow many FinTech/DeFi development firms did this community focused challenge bring into the Cardano ecosystem?"}', -- extra
    'Dewayne Cameron', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    118,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Liqwid:Cardano DeFi Liquidity Pools',  -- title
    'DeFi dApps on Ethereum are the leading dApp sector thats found real product market fit. Users want the ability to generate yield trustlessly',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'f7XhO4+VHG0QLVmIDjYajzxCPe6/VZ+jrrHlcScC8qA=', -- Public Payment Key
    '53000', -- funds
    'http://ideascale.com/t/UM5UZBdlf', -- url
    '', -- files_url
    489, -- impact_score
    '{"solution": "Open source algorithmic & non-custodial liquidity protocol for earning interest on Cardano native assets and borrowing supported assets."}', -- extra
    'Dewayne Cameron', -- proposer name
    '', -- proposer contact
    'https://github.com/Liqwidfinance', -- proposer URL
    'Team implementing has launched several DeFi dApps on Ethereum  

Tweag: Audit/Formal Verification

4ire Labs: DeFi/Fintech development studio', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    119,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Cardano Developer resource portal',  -- title
    'How can we encourage developers and entrepreneurs to build Dapps and businesses on top of Cardano in the next 6 months?',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    'hK3yMszPTkUp/36bkrSdhuQ/qhjGxubVwkodEi1oqTA=', -- Public Payment Key
    '20000', -- funds
    'http://ideascale.com/t/UM5UZBdlS', -- url
    '', -- files_url
    240, -- impact_score
    '{"solution": "A Developer Community portal, to support and provide reference material for developers entering the Cardano ecosystem"}', -- extra
    'Joseph P', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Experienced IT professionals - enterprise systems, applications development and solution architecture', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    120,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Liqwid Developer Portal:Cardano SDK',  -- title
    'DeFi teams need software development kits for dApps & oracles to build extensible use-cases with maximum security & composability on Cardano',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    '0dQxWEIIk/bwI4IPci1h9eCQUyY32ix0B3SKi2AnQBQ=', -- Public Payment Key
    '53000', -- funds
    'http://ideascale.com/t/UM5UZBdlR', -- url
    '', -- files_url
    450, -- impact_score
    '{"solution": "1\\. JavaScript SDK for Cardano & the Liqwid Protocol. The SDK wraps around the **Adrestia JS SDK**  \n2\\. Oracle Open Price Feed SDK  \n3\\. Android SDK"}', -- extra
    'Dewayne Cameron', -- proposer name
    '', -- proposer contact
    'https://github.com/Liqwidfinance', -- proposer URL
    'https://4irelabs.com/financial-software-development/

  

https://www.tweag.io/industry/fintech

  

https://www.ryan-miranda.com', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    121,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Free-Commerce: sell online with ADA',  -- title
    'Enabling stores to sell goods and services for ADA requires intermediaries who have control over your money and charge fees and commissions.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    'NgeiVQY27Y+Oo44hcueHOunm59WPLa1txwV2x43CFFw=', -- Public Payment Key
    '35000', -- funds
    'http://ideascale.com/t/UM5UZBdlN', -- url
    '', -- files_url
    300, -- impact_score
    '{"solution": "To build opensource, intermediary & commission-free payment integrations with ADA & native tokens which flows directly from buyer to seller."}', -- extra
    'Jeronimo Backes', -- proposer name
    '', -- proposer contact
    'https://github.com/uniVocity', -- proposer URL
    'Tech Lead with 20+ years of software development experience, masters degree in computer science, creator of multiple open-source projects.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    122,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Crowdfunding platform with ADA',  -- title
    'Crowdfunding platforms that do not accept payments with cryptos and have no control over the funds.
Many of the financings end in scam.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'dzk12jEZvMz/wB3eCMTMA2H8Jezyvn6AVyWCE006W4U=', -- Public Payment Key
    '42500', -- funds
    'http://ideascale.com/t/UM5UZBdk4', -- url
    '', -- files_url
    244, -- impact_score
    '{"solution": "To release the project funds, investors must authorize them with a Smart Contract , this will allow more transparency and security."}', -- extra
    'Innovatio', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Experience in team management, job planning, market analysis and a leading CEO personality.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    123,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Fandom Auction and Sales Platform',  -- title
    'We would like to completely rebuild our sales platform with a focus on security, stability, and interactivity with the Cardano blockchain.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    '/7j3HlQN/FyE7PkLr6g31HMSgx+eDNALkHOfKL683s4=', -- Public Payment Key
    '25000', -- funds
    'http://ideascale.com/t/UM5UZBdk1', -- url
    '', -- files_url
    207, -- impact_score
    '{"solution": "Fund 3 can assist us in acquiring the necessary resources to hire professionals to code our new platform and write smart contracts."}', -- extra
    'David Kanaszka Jr.', -- proposer name
    '', -- proposer contact
    'https://www.TheDealersDen.com', -- proposer URL
    'Website Management, Community Management, Customer Support, Dispute Resolution, Marketing, Graphic Design, Blockchain Research, Business.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    124,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Marketplace-for-Overstock-Produce',  -- title
    'Food scarcity is an unnecessary plight in modern society. An Est. 40% of food ends up in landfills, impacting prices to environmental issues',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'T/zGqjQJfi62ZdM08KFeyBu5Eo2Y2VHmpoThrTgHjT0=', -- Public Payment Key
    '3000', -- funds
    'http://ideascale.com/t/UM5UZBdkh', -- url
    '', -- files_url
    311, -- impact_score
    '{"solution": "A marketplace for farmers and businesses to buy and sell overstock produce"}', -- extra
    'Joseph Leonard', -- proposer name
    '', -- proposer contact
    'https://github.com/Joseph-Leonard/Marketplace-for-Overstock-Produce', -- proposer URL
    'BS Plant and Soil Science, MS Project Management, 10+ yrs in Agriculture, limited coding and UI.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    125,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'RedToken-Blood Donation dApp',  -- title
    'Nigeria and similar nations lack infrastructure/resources needed to increase physical and financial wellness. **Blood is the resource!**',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'DFOL0/n0p1K00NwgtGF9F+7ynZJIz4VdFsOtqVjfW0s=', -- Public Payment Key
    '15000', -- funds
    'http://ideascale.com/t/UM5UZBdkJ', -- url
    '', -- files_url
    218, -- impact_score
    '{"solution": "Increase life-saving blood supply via **Tokenizing Incentives** for donors by provide access to **++education, health, and financial wellness tools.++**"}', -- extra
    'drkannobeck', -- proposer name
    '', -- proposer contact
    'https://www.redtoken.co', -- proposer URL
    '**++Blood Donor!++** Healthcare, **Tokenomics**, **micro-lending/insurance**, Project Mgmt, Resource Planning, Full Stack Development, Risk Mgmt, **++Nigerian!++**', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    126,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'HOT Potato... the Game!',  -- title
    'We need and awesome game to attract the masses! Here is an **original / simple / fun game**',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'WJqBo7CqzSrj2vwiWkG+D+gbbbBYREdHnsPc7MGBphc=', -- Public Payment Key
    '50000', -- funds
    'http://ideascale.com/t/UM5UZBdkB', -- url
    '', -- files_url
    147, -- impact_score
    '{"solution": "Hot potato is a mobile app game I''ve thought up, that i think will be catchy!\n\nsee \"detailed plan below for more details\""}', -- extra
    'Bryn Fry', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I''m not a developer and i don''t have the time to build this game so i''m looking for someone to build it

I would like to stay on for ideas', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    127,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'OctoWars - a game platform',  -- title
    'Cardano doesn''t provide a simple and elegant way to build trading cards and turn based strategies.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    '8sCkTqtzJBLvXHpTDW7I5ougg/DmplrkV0BcILW6vI8=', -- Public Payment Key
    '125000', -- funds
    'http://ideascale.com/t/UM5UZBdj6', -- url
    '', -- files_url
    154, -- impact_score
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
    128,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Cardano Dapps Listing Website',  -- title
    'As DApps are released on the Cardano network, it will become harder to keep track of projects being released.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    '5ieQIWqGWglzfW/tae305elrcaS4djwoe/fzzLMSpAs=', -- Public Payment Key
    '9500', -- funds
    'http://ideascale.com/t/UM5UZBdju', -- url
    '', -- files_url
    230, -- impact_score
    '{"solution": "We want to develop an application where we can list & give DApp creators a platform to promote dapps, much like Product Hunt, but for Dapps."}', -- extra
    'wiremaven', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Python / Django Developer.

Node JS / Vue JS Developer.

NoSQL and MySQL Experience.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    129,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Online Makerspace',  -- title
    'Currently there are no solution out there with a proper way to incentivize Makers whom explain how something is built.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'O0eVrOhTRmn0ZhhZ7V+KJoWdOAzC+nm5qwxjTJwI/Y4=', -- Public Payment Key
    '25000', -- funds
    'http://ideascale.com/t/UM5UZBdjW', -- url
    '', -- files_url
    206, -- impact_score
    '{"solution": "Through an online makerspace, tightly aligned with the Industry, we are able to create a proper path for the Makers"}', -- extra
    'Roar Holte', -- proposer name
    '', -- proposer contact
    'https://youblob.com', -- proposer URL
    'The project started back in 2012, where the idea won 1st place in Startup Weekend Stavanger.  
In 2018, we started aligning it to the industry', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    130,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Serv Network Professional Profiles',  -- title
    'LinkedIn have a monopoly power over professionals data that gives them an unfair capture of the value created by the professionals.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'iiR+0kvngu9A+2+vsffvqJ3i2w/PZA2a8f8Xn/3Y10A=', -- Public Payment Key
    '3960', -- funds
    'http://ideascale.com/t/UM5UZBdjA', -- url
    '', -- files_url
    328, -- impact_score
    '{"solution": "Self sovereign professional profile that links to protocols and services. Any value gained from linking the data is paid to the user."}', -- extra
    'lovegrovegeorge', -- proposer name
    '', -- proposer contact
    'https://servnetwork.org', -- proposer URL
    'Full stack developer, 5+ years experience, built full stack web and mobile applications from design to implementation.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    131,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Army of Spies-A Market for Secrets',  -- title
    'Many secrets & information asymmetries are unnecessary & hinder society. Transactions to erase them are difficult to organize & execute.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'JG/vW4tnHDe+oOV4SRnok7lFP3heYmBnXqhuGttYKDk=', -- Public Payment Key
    '25000', -- funds
    'http://ideascale.com/t/UM5UZBdiu', -- url
    '', -- files_url
    233, -- impact_score
    '{"solution": "A decentralized marketplace that brokers secrets between Bounty Makers, Intel Sources, & NFT Based Curators."}', -- extra
    'Army of Spies', -- proposer name
    '', -- proposer contact
    'https://github.com/BlocksandBlocks/AOS', -- proposer URL
    'Two giant Cardano fans (a tech company V.P./Attorney & a Fortune 500 Procurement Specialist) dabbling in Marlowe/Plutus for around 2 years.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    132,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Fullcircl: Democracy @ Work',  -- title
    'How can we help online collaborators prosper safely?',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'LWVzr8A7fC//ECE4ANy10UyrXvGjjIvVroJTY2InUEk=', -- Public Payment Key
    '83100', -- funds
    'http://ideascale.com/t/UM5UZBdij', -- url
    '', -- files_url
    205, -- impact_score
    '{"solution": "We believe we can solve the needs of both remote workers and project leaders by creating a trustless system for opt-in work."}', -- extra
    'Chad', -- proposer name
    '', -- proposer contact
    'https://docs.google.com/presentation/d/e/2PACX-1vR8cPx4_BsxnzJLv-E5dDy6UxQgKtk4x1fkh7bQrqKORs-kh7V4L4J8ZB5MjyyAEVHB81v4EkiFcMyv/pub?start=false&loop=false&delayms=5000', -- proposer URL
    'We''re a team of product managers, engineers, and designers with real-world experience developing Enterprise B2B SaaS software products.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    133,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Donating Better with Cardano',  -- title
    'We should trust charities. However, a lot of people don''t know how their donation money is spent thus leading them to giving less.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    '1StTCBXIC7eAIChN5UUa485rPpKz3uSqpqKsJftXaLQ=', -- Public Payment Key
    '42000', -- funds
    'http://ideascale.com/t/UM5UZBdiY', -- url
    '', -- files_url
    248, -- impact_score
    '{"solution": "Bring back trust in charities by using DLT for transparent spending and improve the overall donation experience."}', -- extra
    'Mr Varalta', -- proposer name
    '', -- proposer contact
    'https://igivit.org/how-it-works/', -- proposer URL
    'Not coders but knowledge about DLT and how it can help philanthropy.

Data engineer/Technical PM with 6+ years of exp, Masters degree in CS.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    134,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Ouroboros Networking Rust Crate',  -- title
    'Cardano needs alternative node implementations to increase resilience, networking stack is one of the critical components of such node.',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    'qATehIpnUNfM4HV0kjQ1xqHYMwrBPXzoRLl8MSB8jgU=', -- Public Payment Key
    '7000', -- funds
    'http://ideascale.com/t/UM5UZBdhp', -- url
    '', -- files_url
    327, -- impact_score
    '{"solution": "Develop server-side Ouroboros Networking Protocol as part of our already existing client-side Rust crate / library."}', -- extra
    'mstopka', -- proposer name
    '', -- proposer contact
    'https://github.com/2nd-Layer/rust-cardano-ouroboros-network', -- proposer URL
    'Pavel (Pavlix), the lead developer has years of experience in low-level network stack development, including roles at RedHat.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    135,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Cardano <> Incognito Bridge',  -- title
    'There is no way to anonymously buy, sell or transact ADA on the Cardano network.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'CPyQIpbWM7Moon2+r4cgsSiH2PXzcdEroV1+jhRo2tM=', -- Public Payment Key
    '25000', -- funds
    'http://ideascale.com/t/UM5UZBdhg', -- url
    '', -- files_url
    373, -- impact_score
    '{"solution": "By bridging Cardano with Incognito chain we can add a working solution as a Layer 2 solution without a trusted custodian rather quickly."}', -- extra
    'Wunderbaer Hermes Stakepoool', -- proposer name
    '', -- proposer contact
    'https://github.com/incognitochain/incognito-chain', -- proposer URL
    'The team implementing this has already built bridges for Ethereum, Bitcoin and Binance while Hermespool will support marketing.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    136,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Cardano on Dxsale.network launchpad',  -- title
    'Cardano requires a user-centric crowdsale, locking and listing platform in a trustless way to grow its ecosystem.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'w+D/m6hOw3ouOSTth5dGR/+plVS7C74P1EFXUtYZk+E=', -- Public Payment Key
    '30000', -- funds
    'http://ideascale.com/t/UM5UZBdhI', -- url
    '', -- files_url
    205, -- impact_score
    '{"solution": "https://dxsale.app has already built a user-centric dapp that provides crowdsale, locking and auto listing to Uniswap on ETH chain."}', -- extra
    'Hassan Latif', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'We have successfully built the first iteration of the product for Ethereum at dxsale.app. We will be using our experience to onboard Cardano', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    137,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'The Basket DApp - 売買カート',  -- title
    'Helping communities in COVID times, small local business can''t afford to sell on current online platforms: high fees & long payment terms.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'NgYciCVi5OGmvXnHoWgRDad52wq1X4iDXg7+IcH2H+E=', -- Public Payment Key
    '1', -- funds
    'http://ideascale.com/t/UM5UZBdhG', -- url
    '', -- files_url
    226, -- impact_score
    '{"solution": "Make and experiment DApp with seller/buyer within an existing LOCAL food/farming community, leveraging smart contracts, instant settlements."}', -- extra
    'Patrice', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Knowledgeable on blockchain architectures, Supply Chain/Global Product development professional, local community member.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    138,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Dice Game',  -- title
    'Let''s play some games 😎',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'SZloXc3XHDVG3HLvMsHrDdi0im4UJYtxm+L4so6T9ww=', -- Public Payment Key
    '20000', -- funds
    'http://ideascale.com/t/UM5UZBdhF', -- url
    '', -- files_url
    218, -- impact_score
    '{"solution": "Play a simple mockup version here:\n\nhttps://ada-dice.bubbleapps.io/"}', -- extra
    'Crador', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Here is a budget and timeline plan:
https://drive.google.com/file/d/1sFcN8nxG2Iuba0pDQWsAsOd9lXkCh7M4/view?usp=sharing', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    139,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Decentralized system by staking',  -- title
    'For more better Cardano POS system, How to make ADA holder Decentalization and stable coin system?. ADA whale can destroy Cardano by sellADA',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'QgBvQcQEyYCj4pvm9N1fkT94FOdeAV+iCYTKJ6gc64s=', -- Public Payment Key
    '60000', -- funds
    'http://ideascale.com/t/UM5UZBdg5', -- url
    '', -- files_url
    181, -- impact_score
    '{"solution": "New system for connecting big ADA holder and new (potentially) ADA entrant using Staking advantage and stable coins for ADA Dcentalize."}', -- extra
    'ranket', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'I don''t have any experience for crypto, but I have Ph. D and can understand Cardano deeply. don''t require complicated program.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    140,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Open dApp Auditing Platform',  -- title
    'Sh\*tcoin and scam are everywhere, and investors don''t understand how these protocol works',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'U67hpzD2uE6Szf0lh9yOnTMvNtqLTwjBLQ6WR3CDWtw=', -- Public Payment Key
    '20000', -- funds
    'http://ideascale.com/t/UM5UZBdgz', -- url
    '', -- files_url
    162, -- impact_score
    '{"solution": "An blockchain-based open platform where experts can audit the ICO projects, provide their insight and get reward if their analysis are right"}', -- extra
    'global.anhquan', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Serial Entrepreneur', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    141,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Digital Asset Inheritance',  -- title
    'Inheritance of crypto currencies and other digital assets should not be complicated, expensive or cumbersome to manage.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'ULuvTYWcasWGP/fQx4nPz2Zvlk+6oadLR/+sLBLISQo=', -- Public Payment Key
    '42000', -- funds
    'http://ideascale.com/t/UM5UZBdgl', -- url
    '', -- files_url
    351, -- impact_score
    '{"solution": "A dapp to automate fund transfers in case of death, allowing users to also share seed phrases encoded as \"personal memories\" with loved ones"}', -- extra
    'Jeronimo Backes', -- proposer name
    '', -- proposer contact
    'https://github.com/uniVocity/', -- proposer URL
    'Tech Lead with 20+ years of software development experience, masters degree in computer science, creator of multiple open-source projects.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    142,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Carbonland Trust',  -- title
    'Trees are being cut down at a rapid pace, forests are disappearing, co2 is increasing, the ecosystems that clean our air are in major danger',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'eCRf2QqAwplQZUQGucW72ZlbQnmr8xV2xTTbzdTbKLo=', -- Public Payment Key
    '5000', -- funds
    'http://ideascale.com/t/UM5UZBdgj', -- url
    '', -- files_url
    368, -- impact_score
    '{"solution": "Carbonland Trust''s Forest Conservation Smart Contract and fun blockchain NFT virtual nature preserve game will inspire conservation efforts"}', -- extra
    'Boone Bergsma', -- proposer name
    '', -- proposer contact
    'https://carbonlandtrust.com/', -- proposer URL
    'Recently won best prototype at a hackathon, just had a meeting with 3 leaders from the USDA who pledged support, and secured forest for MVP', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    143,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'DeFi - High Interest Cert of Dep',  -- title
    'Cardano needs economically-attractive Dapp.

Mass retirement capital seeking high yield.

Current DeFi infrastructure vulnerable to exploits',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'JxmJseZXvsN5/fxXeLKA4wq9I/SQeGfjPIMwJboI7GM=', -- Public Payment Key
    '45000', -- funds
    'http://ideascale.com/t/UM5UZBdgh', -- url
    '', -- files_url
    204, -- impact_score
    '{"solution": "Staking Dapp that incentivizes smart-contract-locking with high interest %\n\nIncentivize less ADA on exchanges -->improves ADA performance."}', -- extra
    'Miguel "Why Cardano" ', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    'Investment Manager - 7 years

Producing mediocre Cardano-centric investment content for 19 months.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    144,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'DaPassword - a password manager',  -- title
    'Your credentials stored securely on the cardano blockchain forever. No recurring fees. No central server controlling your data.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'E+IdCTVYZFTjU/AqzmQ2TRcNYmVL4p1Fn8hqi+zGwbI=', -- Public Payment Key
    '48000', -- funds
    'http://ideascale.com/t/UM5UZBdge', -- url
    '', -- files_url
    283, -- impact_score
    '{"solution": "DaPassword is a password manager that stores all your website passwords in the blockchain, along with bookmarks or anything important."}', -- extra
    'Jeronimo Backes', -- proposer name
    '', -- proposer contact
    'https://github.com/uniVocity/', -- proposer URL
    'Tech Lead with 20+ years of software development experience, masters degree in computer science, creator of multiple open-source projects.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    145,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Stiff.Money',  -- title
    'Buying your first home is a challenge for a lot of people. And there are lots of other people with savings that are getting less than 1% APY',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'xHd6/xPoswZiVfWCmtomR0afDVOB/bjOxodcy6PfB0M=', -- Public Payment Key
    '5000', -- funds
    'http://ideascale.com/t/UM5UZBdgd', -- url
    '', -- files_url
    266, -- impact_score
    '{"solution": "Stiff.Money helps first time home buyers work towards homeownership with lease-to-own smart contracts. And provides other people awesome APY"}', -- extra
    'Boone Bergsma', -- proposer name
    '', -- proposer contact
    'https://stiffmoney.com/', -- proposer URL
    'Has used Hard Money to flip two homes for profit, to purchase land for development, and have passed the exam to become a realtor & appraiser', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    146,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Educating Crypto''s Next Generation',  -- title
    'Educating children about cryptocurrencies an securing their future against endless fiat currencies inflation.',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    'UoXzvqSC7EsYqWuzE4LNDhcdkjTNMoGeTqQ77Heau1w=', -- Public Payment Key
    '42065', -- funds
    'http://ideascale.com/t/UM5UZBdgc', -- url
    '', -- files_url
    275, -- impact_score
    '{"solution": "Entertain kids with games that show the benefits of blockchain while providing incentive in the form of real-use tokens to gamers."}', -- extra
    'Chris Ossman', -- proposer name
    '', -- proposer contact
    'https://github.com/fabianaugustus/Cryptomorrow', -- proposer URL
    'As an engineer for the past 20+ years and a developer for the past 2, I bring the experience of working with a team to bring a team together', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    147,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'Wallet Name System (WNS)',  -- title
    'Users should be have the option to send funds to human readable names instead of wallet addresses',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    'KTReCX4M4v0+s8NKO/ucAp5UoWUOGtMWk+ikkILz3HQ=', -- Public Payment Key
    '30000', -- funds
    'http://ideascale.com/t/UM5UZBdgU', -- url
    '', -- files_url
    284, -- impact_score
    '{"solution": "To build a decentralized network that maps names and other information to wallet addresses, much like the internet''s DNS but for wallets."}', -- extra
    'Jeronimo Backes', -- proposer name
    '', -- proposer contact
    'https://github.com/uniVocity/', -- proposer URL
    'Tech Lead with 20+ years of software development experience, masters degree in computer science, creator of multiple open-source projects.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    148,  -- id
    (SELECT row_id FROM objective WHERE id=1 AND event=3), -- objective
    'Grow Africa, Grow Cardano',  -- title
    'How do we seed and grow Cardano in Africa?  
How do we identify and nurture super-spreader proposals across the whole of Africa?',  -- summary
    (SELECT category FROM objective WHERE id=1 AND event=3), -- category - VITSS Compat ONLY
    'G6eDYWc+XCJTQs1cP+Bfyy//A/6fDT8G0xSGShcmevc=', -- Public Payment Key
    '250000', -- funds
    'http://ideascale.com/t/UM5UZBdgT', -- url
    '', -- files_url
    500, -- impact_score
    '{"brief": "**Africa is fertile ground for the adoption of Cardano.**  \nHaving a funding round that focuses on Africa is a win, win, win scenario. Africa wins by getting some backing. Commercial projects that are already building up in Africa win, because they get the signal of support from our community. Cardano holders win when there is an encouraging environment for mass adoption anywhere.  \n  \n\nFocussing on Africa is a strategy that has often been promoted and is openly pursued by IOG and Charles Hoskinson  \n  \n\nThe ROI is likely to be high because community sponsored projects are better run in young and developing economies.  \n\n**What this is:**\n\nA call to activate the Cardano community''s ability to recognise, risk and reward this opportunity to make the world a better place.  \n\n**What this is not:**\n\nThis is not a proposal that needs active work by the proposers, it is a vision that needs votes to become an achievement. If it succeeds the beneficiary is the whole Cardano community. It is a call to close our prejudices and open our eyes, hearts and minds to good people who lack the environmental benefits many of us were born with. Giving them an opportunity to show that circumstances are not something they need charity to rise above and succeed. All that is required is the *equal* opportunity to succeed or fail that many of the rest of us enjoy, but with their hinderances and hurdles lessened and their choices remaining in their own hands.  \n\n**Regardless of whether it succeeds or fails:**\n\nI have had my eyes opened and my mind changed about the potential of Africa. From Africa''s bankers in the west, ex-pats with vision and heart, youth with passions, documentaries that challenged my assumptions and Cardano community members with so much to give.  \n  \n\nThank you to all of you who have commented and interacted with me over the last few weeks.  \n  \n\nhttps://youtu.be/UoOc8FnS7qE  \n\n**Impact:** Fund5 will get underway shortly after the Ethiopian government deal is likely to have been announced and will build on its momentum. Along with this major announcement, Africa has more underutilised talent than this proposal can possibly support but it has unlimited potential to uncover them and get them seen around the world. It is the right time for this community challenge to change the world with maximum effect.\n\n**Feasibility:** Africa is poised for technology focussed growth (see video above), has over 600 technology hubs spread across the continent and is home to 4 of the top 20 fastest growing economies in the world.  \n\n**Auditability:** Like all community proposals it relies on the results of the challenge it proposes to truly prove itself, but the metrics it proposes could give an early indication of how successful it is.\n\n**Budget:** Large enough to attract attention and significant proposals, but small enough to allow other community proposals to seek funding", "importance": "Cardano shines when adoption is global. Africa currently has young entrepreneurs, great growth potential and a government contract imminent.", "goal": "Cardano is considered the best public block-chain operating in Africa\n\nCardano projects in Africa are not isolated but networked and growing", "metrics": "Number of purely African participants  \nNumber of Outside African participants with a significant partnership with African residents  \nFeedback on progress from successful proposals regardless of failure or success  \nNovelty and appropriateness of solutions (hard to quantify, easy to see)  \nNumber of African Fund5 proposals that do not *need* further funding to keep operating\n\nROI of Fund5 community challenge - Grow Africa, grow Cardano"}', -- extra
    'Greg Bell', -- proposer name
    '', -- proposer contact
    '', -- proposer URL
    '', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    149,  -- id
    (SELECT row_id FROM objective WHERE id=3 AND event=3), -- objective
    'West Africa Decentralized Alliance',  -- title
    'West African Developers, Academia & Governments are unaware of Cardano''s potential to solve local problems',  -- summary
    (SELECT category FROM objective WHERE id=3 AND event=3), -- category - VITSS Compat ONLY
    '5tGFpFD3d8dnkZAxiHbwkNyqXOEgP1OvqISowIg6eeA=', -- Public Payment Key
    '31885', -- funds
    'http://ideascale.com/t/UM5UZBdgO', -- url
    '', -- files_url
    362, -- impact_score
    '{"solution": "Create a network of Cardano **Resource Hubs for Solutions Design** for local influence and developer onboarding throughout West Africa"}', -- extra
    'Mercy', -- proposer name
    '', -- proposer contact
    'https://wadalliance.org/', -- proposer URL
    'Team: Marketing, Project & Nonprofit Work Experience in W/Africa, Software Developer, Project Management, Analytics, Community Engagement', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)
,

(
    150,  -- id
    (SELECT row_id FROM objective WHERE id=2 AND event=3), -- objective
    'Crowdano - Crowdfunding Platform',  -- title
    'Backers of a crowdfunding campaign should be able to ensure their funds are used correctly, however 9% of Kickstarters fail to deliver\[1\]\[2\]',  -- summary
    (SELECT category FROM objective WHERE id=2 AND event=3), -- category - VITSS Compat ONLY
    '2/7Uoocv0en8g+3Y9b3npTocbIHEXgXTPGIRv3HbLJg=', -- Public Payment Key
    '24030', -- funds
    'http://ideascale.com/t/UM5UZBdgJ', -- url
    '', -- files_url
    378, -- impact_score
    '{"solution": "I propose a crowdfunding platform based on Cardano which implements a staged-funding model to protect backer''s investments."}', -- extra
    'Edward Jane', -- proposer name
    '', -- proposer contact
    'https://github.com/ejane24/crowdano', -- proposer URL
    'Over 7 years experience in software development with languages including, but not limited to; Python, Solidity, Javascript, HTML, and PHP.', -- relevant experience
    'None',  -- bb_proposal_id
    '{ "yes", "no" }' -- bb_vote_options - Deprecated VitSS compat ONLY.
)

;

