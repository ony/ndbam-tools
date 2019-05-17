Feature: Report type of objects mismatches

    Background:
        Given sample with minimum content

    Scenario: File instead of symlink
        Given file /not-a-symlink
        And file /var/db/ndbam/data/dummy/0:0/contents
            """
            type=sym path=/not-a-symlink target=dummy mtime=1430338107
            """
        When run ndbam-check --allow-mtime --no-integrity
        Then output is:
            """
            dummy-0:0
              T /not-a-symlink Not a symbolic link
              # Size: 0 B
            """
        And failure

    Scenario: File instead of directory
        Given file /not-a-dir
        And file /var/db/ndbam/data/dummy/0:0/contents
            """
            type=dir path=/not-a-dir
            """
        When run ndbam-check --allow-mtime --no-integrity
        Then output is:
            """
            dummy-0:0
              T /not-a-dir Not a directory
              # Size: 0 B
            """
        And failure

    Scenario: Directory instead of file
        Given directory /not-a-file
        And file /var/db/ndbam/data/dummy/0:0/contents
            """
            type=file path=/not-a-file md5=00000000000000000000000000000000 mtime=1430338107
            """
        When run ndbam-check --allow-mtime --no-integrity
        Then output is:
            """
            dummy-0:0
              T /not-a-file Not a regular file
              # Size: 0 B
            """
        And failure
