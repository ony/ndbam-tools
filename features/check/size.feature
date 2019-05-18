Feature: We can use ndbam-check to verify size of package or total

    Background:
        Given sample with basic content

    Scenario: Size report for empty package
        When run ndbam-check --allow-mtime --no-integrity --show-size empty
        Then success
        And output is:
            """
            empty-0:0
              # Size: 0 B
            """

    Scenario: Size report for good package
        When run ndbam-check --allow-mtime --no-integrity --show-size hello
        Then success
        And output is:
            """
            hello-0:0
              # Size: 20 B

              # Total size: 20 B
            """

    Scenario: Report size for package with amended content (most checks disabled)
        When run ndbam-check --allow-mtime --no-integrity --show-size amended
        Then success
        And output is:
            """
            amended-0:0
              # Size: 8 B

              # Total size: 8 B
            """

    Scenario: Amended content detected and not counted toward size
        When run ndbam-check --allow-mtime --show-size amended
        Then output is:
            """
            amended-0:0
              C /amended.txt Content changed
              # Size: 0 B
            """
            # It is not intended to not report total size here

    Scenario: Package with symlink and directories only
        Given dir /lib
        And symlink /lib64 to lib
        And file /var/db/ndbam/data/no-files/0:0/contents
            """
            type=dir path=/lib
            type=sym path=/lib64 target=lib mtime=0
            """
        When run ndbam-check --allow-mtime --no-integrity --show-size no-files
        Then success
        And output is:
            """
            no-files-0:0
              # Size: 0 B
            """
