# cower - **CO**ntainer **W**ak**ER**

I have no idea how one would even set
[Wake-on-LAN](https://en.wikipedia.org/wiki/Wake-on-LAN) for containers up. So,
this is what I created specifically for this purpose - waking containers up
remotely.

One might (quite reasonably) object that there are things like Red Hat's
[Cockpit](https://cockpit-project.org/), or that the same thing can be achieved
using just SSH.

One would be correct. However, setting up some complicated permissions system or
giving your colleagues (or friends, whoever) full SSH/Cockpit access to your
(company's) server is bad practice. I for one wouldn't trust my friends with
that kind of power.

## My use case

Have you ever played Minecraft with friends on [Aternos](https://aternos.org)?
The service is great, but what if you want to host your own Minecraft server, on
your own hardware?

I have a Proxmox VE with a Minecraft Fabric container, and I want to let my
friends start the container if I stop it or if it crashes. This is what `cower`
does!

## Architecture

<!-- TODO add graph to visualize communication between components -->

- Client - someone who wants to remotely wake containers
- Server - something that forwards commands to targets, although not always
           necessary (in case of public IPv4 or non-NATed IPv6 addresses)
- Target - the server that actually runs the containers

Servers aren't strictly necessary. If you aren't behind a NAT, you should be
just fine routing `cower` commands straight from `Client`s to `Server`s.

## Protocol

Cower uses its custom protocol. See [PROTOCOL.md](PROTOCOL.md) for more information.

## The `test-keys` directory

**The `test-keys/` directory contains keys used for testing, as the name
suggests. They are most definitely NOT TO BE USED IN PRODUCTION!!!** Generate
your own keys, people.

The keys are RSA, but that doesn't mean that you can't use any other algorithm.
In fact, post-quantum algos should almost always be preferred over good
old RSA.
