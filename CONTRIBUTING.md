# Contributing
copes welcomes your contributions.

## Issues writing guidelines
Please, follow these guidelines to write your issues. Reported issues not following this guidelines may be closed as invalid.

* One issue per report.
* Be precise and clear.
* Use the issue templates.
* Use good descriptive titles for new issues.

Please, as first step, search for similar issues on the issue tracker. If you find an already opened issue and you can provide new information, then add a new comment (please, do not write +1 o similar comments). If the issue is closed, open a new one and include a reference to the old one (for example, add 'Related to #<closed_issue_number>').

## Merge request
Before starting to work on a merge request, please follow these instructions:

1. Open an issue explaining the reason for the change, state that you want to work on it and wait for the developers response.
2. Create a merge request from a **new branch** (do not create the MR from the master branch) and link it with the issue.  The merge request must be focused on only one feature or topic.
3. Follow the [coding style](#coding-style) rules and make sure that your code integrates well into the application architecture.
4. If you are working on new code, make it testable when possible. Non trivial domain code must be tested.
5. Commit your work on atomic unit of changes. If you are unsure of how to do it, check the commit history for examples of such commits.
6. Explain the changes introduced by non straightforward commits on their commit message.
7. During the merge request, be concise on your comments and make sure that you fully understand what you are stating on them. Take your time and, when needed, do your research before posting.

## Coding style
* Format your code with [rustfmt](https://github.com/rust-lang/rustfmt).
