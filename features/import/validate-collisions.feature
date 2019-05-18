Feature: Collision protection from the box

    Scenario: Existing file
        Given file /collider
        And file /tmp/image/collider
        When implemented
        When run ndbam-import --location /tmp/image sci-physics/particles
        Then failure

    Scenario: Existing directory
        Given dir /umbrella
        And dir /tmp/image/umbrella
        When implemented
        When run ndbam-import --location /tmp/image sec-policy/wong
        Then failure

    Scenario: Existing symlink with missing target
        Given symlink /back to future
        And dir /tmp/image/back
        When implemented
        When run ndbam-import --location /tmp/image sci-misc/el
        Then failure

    Scenario: Existing symlink to directory
        Given symlink /hint to world
        And dir /tmp/image/hint
        When implemented
        When run ndbam-import --location /tmp/image app-misc/answers
        Then failure
