Feature: Besides modification time and checksum we also support symlink checks

    Background:
        Given sample with minimum content

    Scenario: Symlink without changes
        Given file /target
        And symlink /symlink to target
        And file /var/db/ndbam/data/dummy/0:0/contents
            """
            type=sym path=/symlink target=target mtime=1430338107
            """
        When run ndbam-check --allow-mtime --no-integrity
        Then success
        And no output

    Scenario: Symlink pointing to wrong target
        Given file /target
        And symlink /symlink to target
        And file /var/db/ndbam/data/dummy/0:0/contents
            """
            type=sym path=/symlink target=wrong-target mtime=1430338107
            """
        When run ndbam-check --allow-mtime --no-integrity
        Then failure
        And output is:
            """
            dummy-0:0
              C /symlink Symlink changed
              # Size: 0 B
            """

    Scenario: Symlink pointing to absent object
        Given symlink /dangling to missing
        And file /var/db/ndbam/data/dummy/0:0/contents
            """
            type=sym path=/dangling target=missing mtime=1430338107
            """
        When run ndbam-check --allow-mtime --no-integrity
        Then failure
        And output is:
            """
            dummy-0:0
              X /dangling Dangling symbolic link
              # Size: 0 B
            """
