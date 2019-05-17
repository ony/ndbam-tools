Feature: Check content integrity using checksum unless --no-integrity specified

    Scenario: Good package
        Given sample with basic content
        When run ndbam-check --allow-mtime hello
        Then success
        And no output

    Scenario: Package with amended files
        Given sample with basic content
        When run ndbam-check --allow-mtime amended
        Then output is:
            """
            amended-0:0
              C /amended.txt Content changed
              # Size: 0 B
            """
        And failure

    Scenario: Package with amended files and --no-integrity flag
        Given sample with basic content
        When run ndbam-check --allow-mtime --no-integrity amended
        Then success
        And no output
