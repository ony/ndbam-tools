Feature: Identify modification time change (mtime)

    Background:
        Given sample with minimum content

    Scenario: None of the files for package were modified since installation
        Given mtime for file /note.txt is <M1>
        And file /var/db/ndbam/data/dummy/0:0/contents
            """
            type=file path=/note.txt md5=00000000000000000000000000000000 mtime=<M1>
            """
        When run ndbam-check --no-integrity
        Then success
        And no output

    Scenario: None of the files for package were modified since installation
        Given mtime for file /note.txt is <M1>
        And mtime <M2> 60 seconds in past
        And file /var/db/ndbam/data/dummy/0:0/contents
            """
            type=file path=/note.txt md5=00000000000000000000000000000000 mtime=<M2>
            """
        When run ndbam-check --no-integrity
        Then failure
        And output is:
            """
            dummy-0:0
              M /note.txt Modification time changed
              # Size: 0 B
            """
