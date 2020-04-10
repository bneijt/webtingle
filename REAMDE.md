Webtingle
========

    Let the servers feel each other

Simple web app to HTTP call urls every few seconds and show an access log of calling parties.

Meant to be a demo app for a simple runtime state application where you can check access and firewall rules.

    systemfd --no-pid -s http::3000 -- cargo watch -x run