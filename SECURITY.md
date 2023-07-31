# Security Policy

## Supported Versions

The main tagged versions of the app should be the only ones deployed on the mainnet network.

| Version | Supported          |
| ------- | ------------------ |
| 1.x.x   | :white_check_mark: |
| 0.x.x   | :x:                |

## About Reporting a Vulnerability

According to the [Github coordinated disclosure](https://docs.github.com/en/code-security/security-advisories/guidance-on-reporting-and-writing/about-coordinated-disclosure-of-security-vulnerabilities#about-disclosing-vulnerabilities-in-the-industry) 

> Vulnerability disclosure is an area where collaboration between vulnerability reporters, such as security researchers, and project maintainers is very important. Both parties need to work together from the moment a potentially harmful security vulnerability is found, right until a vulnerability is disclosed to the world, ideally with a patch available. Typically, when someone lets a maintainer know privately about a security vulnerability, the maintainer develops a fix, validates it, and notifies the users of the project or package.

> The initial report of a vulnerability is made privately, and the full details are only published once the maintainer has acknowledged the issue, and ideally made remediations or a patch available, sometimes with a delay to allow more time for the patches to be installed. For more information, see the ["OWASP Cheat Sheet Series about vulnerability disclosure"](https://cheatsheetseries.owasp.org/cheatsheets/Vulnerability_Disclosure_Cheat_Sheet.html#commercial-and-open-source-software) on the OWASP Cheat Sheet Series website.

## Best Practices According to Github

According to the [Github coordinated disclosure](https://docs.github.com/en/code-security/security-advisories/guidance-on-reporting-and-writing/about-coordinated-disclosure-of-security-vulnerabilities#best-practices-for-maintainers) 

> It's good practice to report vulnerabilities privately to maintainers. When possible, as a vulnerability reporter, we recommend you avoid:

- Disclosing the vulnerability publicly without giving maintainers a chance to remediate.
- Bypassing the maintainers.
- Disclosing the vulnerability before a fixed version of the code is available.
- Expecting to be compensated for reporting an issue, where no public bounty program exists.

It's acceptable for vulnerability reporters to disclose a vulnerability publicly after a period of time, if they have tried to contact the maintainers and not received a response, or contacted them and been asked to wait too long to disclose it.

## Process for Shade Protocol

Although Github security reports are available for the main Shade Repository, we will follow custom reporting procedure so that the reports get submitted diretly to the relevant teams in the Shade organization. This will allow us to have a more streamlined process for handling reports across all the different repositories.

Most of the reports will be handled by the [Secure Secrets Security Team](security@securesecrets.org) and the reports have to be submitted here: [Official Vulnerability Disclosure Portal](https://securesecrets.atlassian.net/servicedesk/customer/portal/3/group/11/create/37). When a security incident is reported, the user reporting accepts the terms and conditions of the Bounty Program, and is automtically enrolled into the Bounty Program detailed in the [Shade Protocol Responsible Disclosure](./Shade_Protocol_Resposible_Disclosure.md). The user can request to be not part of the Bounty Program by sending an email follow up to the initial report, but still needs to follow the process of the Github Coordinated Disclosure and Best Practices detailed by Github above.


