# Shade Protocol Responsible Disclosure Policy

Shade Protocol is an interconnected suite of privacy preserving dApps built on the Secret Network whose smart contracts leverage the privacy preserving properties of secret contracts in order to empower private DeFi. The Shade Protocol Responsible Disclosure Framework establishes and defines the following for the Shade Security Policy:
- Vulnerability Severity Classification System (VSCS)
- Rewards By Threat Level
- Assets in Scope
- Out of Scope Work
- Proof of Concept (PoC) guidelines
- Rules of Engagement
- SLA Response Times
- Security Patch Policy and Procedure
- Rewards Payment Process

For general information about reporting vulnerabilities, visit [security policy overview](./SECURITY.md). The Shade Protocol Responsible Disclosure Framework is subject to change at the discretion of the Shade Protocol core team.


## Bug Bounty Program Classes
- **Smart Contracts**
- **Website and Applications**
- **Infrastructure**

## Vulnerability Severity Classification System

### Smart Contracts

#### **Critical**

- Direct theft of any user funds 
- Permanent Freezing of funds 
- Protocol Insolvency 
- Governance Voting Result Manipulation

#### **High**

- Theft of unclaimed yield
- Permanent freezing of unclaimed yield
- Temporary freezing of funds

#### **Medium** 
- Smart contract unable to operate due to lack of token funds
- Theft of gas
- Unbounded gas consumption
- Griefing (damage to users or protocol with no profit motive)

#### **Low**
Contract fails to deliver promised returns, but doesn’t lose value


### Website and Applications

#### **Critical**
- Taking down the website/application
- Direct theft of any user funds
- Execute arbitrary system commands
- Retrieve sensitive data from a server
- Taking state-modifying authenticated actions on behalf of other users with any interaction by that users.
- Subdomain takeover with already connected wallet interaction
- Malicious interactions with an already-connected wallet

#### **High**
- Improperly disclosing confidential user information
- Changing sensitive details of other users (including modifying browser local storage) without already-connected wallet interaction.
- Injecting/modifying the static content on the target application without Javascript (Persistent) such as:
    - HTML injection without Javascript
    - Replacing existing text with arbitrary text
    - Arbitrary file uploads, etc.

#### **Medium** 
- Injecting/modifying the static content on the target application without Javascript (Reflected) such as:
        - Reflected HTML injection
        - Loading external site data
- Redirecting users to malicious websites (Open Redirect)

#### **Low**
- Taking over broken or expired outgoing links
- Disabling access to site for users
- Preventing connection to wallet
- Cookie bombing


### Infrastructure
#### **Critical**
 - Access to any central keys address controlled by the project (e.g. private keys, seed phrases, etc.)
 - Access to Central Database

#### **High**
 TBD

#### **Medium** 
 TBD

#### **Low**
 TBD

## Rewards by Threat Level

### Smart Contract
| Vulnerability | Reward | Requirements |
| ------------ | ----------------- | --------------------- |
| Critical | ***Payout: Up to 40k SHD*** | PoC Requirement |
| High | Payout: Up to 5k SHD | PoC Requirement |
| Medium | Payout: Up to 1K SHD | PoC Requirement |
| Low | Payout: Up to 100 SHD | PoC Requirement |


### Website and applications

| Vulnerability | Reward | Requirements |
| ------------ | ----------------- | --------------------- |
| Critical | ***Payout: Up to 10k SHD*** | PoC Requirement |
| High | Payout: Up to 1k SHD | PoC Requirement |
| Medium | Payout: Up to 100 SHD | PoC Requirement |
| Low | Payout: Up to 10 SHD | PoC Requirement |

### Infrastructure

| Vulnerability | Reward | Requirements |
| ------------ | ----------------- | --------------------- |
| Critical | ***Payout: Up to 10k SHD*** | PoC Requirement |
| High | Payout: Up to 1k SHD | PoC Requirement |
| Medium | Payout: Up to 100 SHD | PoC Requirement |
| Low | Payout: Up to 10 SHD | PoC Requirement |


Critical smart contract vulnerabilities are capped at 10% of economic damage, primarily taking into consideration the funds at risk. In cases of repeatable attacks, only the first attack is considered unless the smart contract cannot be upgraded or paused. High smart contract vulnerabilities will be capped at up to 100% of the funds affected.

Critical website and application bug reports will be rewarded with 10k SHD only if the impact leads to a direct loss in funds or a manipulation of the votes or the voting result, as well as the modification of its display leading to a misrepresentation of the result or vote. All other impacts that would be classified as Critical would be rewarded 1K SHD.

All calculations of the amount of funds at risk are done based on the time the bug report is submitted.

### Response Times:

| Action | Severity Level | Response Time |
| ------------ | ----------------- | --------------------- |
| Acknowledgment of report | Critical | 48 hours |
| | All severity levels except critical | 3-4 days (working days) | 
| Processing of Report |  Critical + High | Up to 14 days |
| | Medium + Low | Up to 14 days |
| Payout for valid reports | Critical + High | Within 14 days |
| | Medium + Low | Within 14 days |



## Assets considered in scope:

[Shade Protocol Audit Log](https://docs.shadeprotocol.io/shade-protocol/research/audit-log)

### **Smart Contracts:**

#### Shade Oracle - https://github.com/securesecrets/shade-oracle 

- https://github.com/securesecrets/shade-oracle/tree/release/contracts/oracle_router
- https://github.com/securesecrets/shade-oracle/tree/release/contracts/index_oracle
- https://github.com/securesecrets/shade-oracle/tree/release/contracts/shade_staking_derivatives_oracle 
- https://github.com/securesecrets/shade-oracle/tree/release/contracts/shadeswap_market_oracle
- https://github.com/securesecrets/shade-oracle/tree/release/contracts/shadeswap_spot_oracle
- https://github.com/securesecrets/shade-oracle/tree/release/contracts/stride_staking_derivatives_oracle
- https://github.com/securesecrets/shade-oracle/tree/release/contracts/siennaswap_market_oracle



#### ShadeSwap - https://github.com/securesecrets/shadeswap 

- https://github.com/securesecrets/shadeswap/tree/main/contracts/amm_pair
- https://github.com/securesecrets/shadeswap/tree/main/contracts/factory
- https://github.com/securesecrets/shadeswap/tree/main/contracts/lp_token
- https://github.com/securesecrets/shadeswap/tree/main/contracts/router
- https://github.com/securesecrets/shadeswap/tree/main/contracts/snip20
- https://github.com/securesecrets/shadeswap/tree/main/contracts/staking

#### Shade Protocol - https://github.com/securesecrets/shade 

- https://github.com/securesecrets/shade/blob/main/contracts/governance
- https://github.com/securesecrets/shade/blob/main/contracts/staking
- https://github.com/securesecrets/shade/blob/main/contracts/scrt_staking
- https://github.com/securesecrets/shade/blob/main/contracts/treasury
- https://github.com/securesecrets/shade/blob/main/contracts/mint
- https://github.com/securesecrets/shade/blob/main/contracts/oracle 
- https://github.com/securesecrets/shade/blob/main/contracts/airdrop

*In order to be eligible for a reward, the vulnerability must exist in both the deployed contract and its respective Github repository.*

#### Website and Applications
- https://shadeprotocol.io
- https://app.shadeprotocol.io


## Out of Scope Work

### Smart Contracts:

- Basic Governance attacks
- Sybil attacks
- Centralization risks
- Disruption of price feeds from third party oracles (does not include oracle manipulation/flash loan attacks)

### Website and Applications
- CSRF with no security impact
- Theoretical vulnerabilities without any proof or demonstration
- Missing HTTP Security Headers or cookie security flags
- Server-side information disclosure such as IPs, server names, and most stack traces
- URL Redirects (unless combined with another vulnerability to produce a more severe vulnerability)
- DDoS vulnerabilities
- Feature requests
- Best practices
- DNS Sabotage
- Self-XSS
- Captcha bypass using OCR
- Vulnerabilities used to enumerate or confirm the existence of users or tenants
- Vulnerabilities requiring unlikely user actions
- Lack of SSL/TLS best practices
- Attacks requiring privileged access from within the organization
- Vulnerabilities primarily caused by browser/plugin defects
- Any vulnerability exploit requiring CSP bypass resulting from a browser bug


### The following vulnerabilities are excluded from the rewards for this bug bounty program:

- Previously-discovered bugs
- Attacks that the reporter has already exploited themselves, leading to damage
- Attacks requiring access to leaked keys/credentials
- Attacks requiring access to privileged addresses (governance, strategist)
- Attacks leveraging other DeFi protocols, unless the following are true:
    - Losses or negative effects of the attack impact Shade Protocol ecosystem participants including SHD and SILK token holders
    - Additional DeFi protocols used exist as smart contracts on the Secret Network mainnet and can reasonably be expected to have enough liquidity in various assets to allow the attack to succeed.

### Proof of Concept (PoC) Guidelines:
- The smart contract PoC should always be made by forking the mainnet
- The PoC should contain runnable code for the exploit demonstration
- The PoC should mention all the dependencies, configuration files, and environmental variables that are required in order to run that PoC, as any other requirements to run the test.
    - Add clear and descriptive replication steps so that the Shade Protocol Team can easily reproduce and validate your findings.
-  PoCs should have clear print statements and or comments that detail each step of the attack and display relevant information, such as funds stolen/frozen etc.
- The PoC should ideally determine and provide data on the amount of funds at risk, which can be determined by calculating the total amount of tokens multiplied by the average price of the token at the time of the submission.
- The PoC must comply with any additional guidelines specified by the bug bounty program the whitehat is submitting a bug report to. 

## Rules of Engagement
*Violation of these rules can result in a temporary suspension or permanent ban from the Shade Protocol Bug Bounty Program, which may result in the forfeiture of potential payout and loss of access to all bug submissions. Shade Protocol has a zero tolerance policy for spam/incomplete bug reports and misrepresentation of bug severity.*

The Shade Protocol team will take all reasonable actions to ensure the successful execution of and the maximum effectiveness of the Shade Protocol Bug Bounty Program. 

### Standard Program Rules:
- Unless otherwise noted, users should create accounts for testing purposes.
- Submissions must be made exclusively through the [Official Vulnerability Disclosure Portal](https://securesecrets.atlassian.net/servicedesk/customer/portal/3/group/11/create/37) to be considered for a reward.
- Communication regarding submissions must remain within Shade Protocol Bug Bounty support channels for the duration of the disclosure process.
- Users must submit a Proof of Concept (PoC) in order to receive bounties for bug reports.
	- Example PoC can be found in the [Shade Security Advisories](https://github.com/securesecrets/shade/security/advisories).

### Prohibited Behaviors:
- Any testing with mainnet contracts. Testing on mainnet is grounds for an immediate and permanent ban
- Misrepresenting assets in scope: claiming that a bug report impacts/targets an asset in scope when it does not
- Misrepresenting severity: claiming that a bug report is critical when it clearly is not
- Automated testing of services that generates significant amounts of traffic
- Attempting phishing or other social engineering attacks against Shade Protocol or its team members.
- Harassment, i.e., excessive, abusive, or bad faith communication
- Disputing a bug report in the dashboard once it has been paid or marked as closed
- Impersonation
- Threatening to publish or publishing personal information without consent
- Reporting a bug that has already been publicly disclosed
- Publicly disclosing a bug report--or even the existence of a bug report for a specific project--before it has been fixed and paid
- Publicly disclosing a bug report before 30 days have elapsed since the project closed the report as being out of scope or not requiring a fix
- Publicly disclosing a bug report deemed to be a duplicate or well-known to the project
- Submitting spam/very low-quality bug reports and submitting information through our platform that is not a bug report
- Submitting AI-generated/automated scanner bug reports
- Submitting fixes to a project's repository without their express consent
- Unauthorized disclosure or access of sensitive information beyond what is necessary to submit the report.

## Security Patch Policy and Procedure

All valid security bugs will be handled according to the following guidelines and will trigger an internal incident response process. We will keep you updated and work with you through the process. 

Even if you are not eligible for the bounty program you are recommend to follow all security patch procedures as highlighted in the [security policy overview](./SECURITY.md).

Security patches are very sensitive by nature, and their exposure can provide a window of opportunity for a malicious actor to attack the protocol or any of its applications before it has been patched. As a result, Shade Protocol has adopted the following policies:

- Due to the sensitive nature of security patches, the Shade Protocol team will **not** make the content of security patches public until the entire system has been patched.
- Users submitting bug reports are expected **not** to publicly disclose a bug report--or even the existence of a bug report for a specific project--before it has been fixed.
- Users submitting bug reports are expected **not** to publicly disclose a bug report deemed to be a duplicate or well-known to the Shade Protocol team.
- The details of the patch will be revealed after a holdover period of 10 days from the time after the successful application of patches to relevant systems. 
    - The holdover period provides reasonable time to ensure the stability of the contract or application before patch is revealed.
    - The patch activity is considered successful if the applied patch completely mitigated the vulnerability as intended and the system remained stable.

## Rewards Payment Process

Users must have access to a Secret Network wallet address to receive any earned Bug Bounty rewards. Once a Bug Bounty report has been submitted, received, and verified, the Shade Protocol team will reconfirm the severity, reward payout amount, and wallet address to send bug bounty with the user who submitted the bug report. 

Once the bug report has been fully processed and the patch resulting from the Bug Bounty report has been successfully applied, the bug report will be considered “Closed”, at which time users will be notified via their provided email that they may begin the KYC process for the wallet receiving funds. 
