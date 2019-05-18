Feature: Compatibility with Paludis in package/content registry formats

    Background:
        Given sample with minimum content
        And directory /usr/share/ca-certificates/mozilla
        And semi-binary file /var/db/ndbam/data/app-misc---ca-certificates/20190110:0/contents
            """
            type=dir path=/usr
            type=dir path=/usr/share
            type=dir path=/usr/share/ca-certificates
            type=dir path=/usr/share/ca-certificates/mozilla
            type=file path=/usr/share/ca-certificates/mozilla/NetLock_Arany_\\=Class_Gold\\=_F\\\xc5\\\x91tan\\\xc3\\\xbas\\\xc3\xadtv\\\xc3\\\xa1ny.crt md5=22f5bca8ba618e920978c24c4b68d84c mtime=1549752020
            """

    Scenario: Package content registry with UTF-8 bytes escaped
        Given file /usr/share/ca-certificates/mozilla/NetLock_Arany_=Class_Gold=_Főtanúsítvány.crt
        When run ndbam-check --allow-mtime --no-integrity app-misc/ca-certificates
        Then success
        And no output

    Scenario: Package content registry with UTF-8 bytes escaped pointing to non-existing file
        When run ndbam-check --allow-mtime --no-integrity app-misc/ca-certificates
        Then failure
        And output is:
            """
            app-misc/ca-certificates-20190110:0
              X /usr/share/ca-certificates/mozilla/NetLock_Arany_=Class_Gold=_Főtanúsítvány.crt Does not exist
              # Size: 0 B
            """
