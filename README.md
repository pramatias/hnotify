Hnotify is a notification daemon for Ycombinator, it monitors and notifies about comments to a user.

Dependencies:
    <b>notify-send</b>

notify-send is a linux cli command and uses that to send notifications.

It requires a mysql database, to keep track of the comments.

Running it for the first time:

<b>hnotify --init</b>

Asks the user once for some configuration data:
hn username:
db username:
db password:
db port:
and how many seconds to check for new comments

It creates a file to store the data, ~/.hnotifyrc

In case some configuration needs to be setup again, hnotify --init will overwrite it.

As soon as the initilization is complete: <b> hnotify & </b>

The code is almost 100% computer generated.

Conversation with GPT:
https://chat.openai.com/share/db88b27f-c697-4dcd-a140-37831f5c62af
https://chat.openai.com/share/ec61ef8c-c90d-41e6-9de0-60d24e996d64

License MIT
