# bin
a paste bin.

This is a fork of [w4](https://github.com/w4/)'s [bin](https://github.com/w4/bin) project.  
The fork changes 2 aspects of the original:

##### 1. Paste IDs
This version of bin uses a 12 character long randomly generated string of upper or lower case letters. The letters I, L and O are excluded from IDs since they are easily confused for other symbols. This is simply for my own preference, I like the look of the URLs better this way.

##### 2. Paste storage
This version saves pastes to disk, in order for them to persist even if the service restarts. The `--paste-dir` argument can be used to define the directory in which the pastes are stored. In the provided docker image the directory `/srv/pastes` is used, and should probably be volume mounted to true persistence. Note that the user that runs bin now has the user ID `1000`, so that it is easy to give correct ownership to the mounted directory.

Since pastes now save to disk, the `--buffer-size` argument has been removed. If you still want to remove old pastes as new pastes are created, use a cron job or similar to purge old files.


For more information, see [w4](https://github.com/w4/)'s [repository](https://github.com/w4/bin).
