Feature: Correctness of symlinks proven before actual install

    Scenario: Relative symlink that crosses root
        Given symlink /tmp/image/runaway to ../the-13th-floor
        When implemented
        When run ndbam-import --image /tmp/image virtualization/world
        Then failure

    Scenario: Absolute symlinks that is valid relative to root
        Given file /tmp/image/hole/club
        And symlink /tmp/image/white-rabbit to /hole/club
        When implemented
        When run ndbam-import --image /tmp/image sys-power/club
        Then success

    Scenario: Dangling symlink
        Given file /tmp/build/libmy.so
        And symlink /tmp/image/libmy.so to /tmp/build/libmy.so
        When implemented
        When run ndbam-import --image /tmp/image dev-libs/loosy
        Then failure
