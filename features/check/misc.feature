Feature: Miscelanous properties of ndbam-check

    Scenario: There is nothing to check
        Given sample with minimum content
        When run ndbam-check --verbose
        Then success
        And no output

    Scenario: Request to check package that were not installed
        Given sample with minimum content
        When run ndbam-check --show-size not-installed
        Then output is:
            """
            not-installed - Not found
            """
        And failure
