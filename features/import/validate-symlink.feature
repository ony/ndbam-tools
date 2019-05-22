Feature: Correctness of symlinks proven before actual install

    Background:
        Given sample with minimum content

    Scenario: Relative symlink that crosses root
        Given symlink /tmp/image/runaway to ../the-13th-floor
        When implemented
        When run ndbam-import --image ${root}/tmp/image virtualization/world
        Then failure

    Scenario: Absolute symlinks that is valid relative to root
        Given file /tmp/image/hole/club
        And symlink /tmp/image/white-rabbit to /hole/club
        When run ndbam-import --image ${root}/tmp/image sys-power/club
        Then success
        And symlink /white-rabbit to /hole/club exists

    Scenario: Relative cross-link (infinite loop)
        Given symlink /tmp/image/flip to flop
        Given symlink /tmp/image/flop to flip
        When run ndbam-import --image ${root}/tmp/image app-misc/flipflop
        Then failure
        And no symlink /flip exists
        And no symlink /flop exists

    Scenario: Dangling relative symlink
        Given symlink /tmp/image/libmy.so to .build/libmy.so.1
        When run ndbam-import --image ${root}/tmp/image dev-libs/relatively-loosy
        Then failure

    Scenario: Dangling absolute symlink pointing outside of image folder
        Given file /tmp/build/libmy.so
        And symlink /tmp/image/libmy.so to /nonexistent/libmy.so
        When run ndbam-import --image ${root}/tmp/image dev-libs/absolutely-loosy
        Then failure

    Scenario: Dangling absolute symlink pointing into image folder
        Given file /tmp/image/libmy.so.1
        And symlink /tmp/image/libmy.so to /tmp/image/libmy.so
        When run ndbam-import --image ${root}/tmp/image dev-libs/absolutely-loosy
        Then failure
