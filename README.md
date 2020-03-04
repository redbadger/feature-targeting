# Feature targeting

Infrastructure support to enable feature targeting in microservices based web
systems (and mobile apps).

This is currently highly experimental and production
use is not advisable unless you're brave.

## Getting started

See the [samples](./samples/README.md) directory to try some examples.

## About feature targeting

Feature targeting is an evolution (or a clarification) of the capability commonly
refered to as [feature toggles](https://www.martinfowler.com/articles/feature-toggles.html).

The terms "feature toggle"is a little misleading. Partly in metioning "features",
leading to an assumption this capability is only useful for customer facing
product changes, but mostly due to the assumption that features can be on or
off for everyone and there is nothing in between.

_The key wrong assumption being made is that the entire audience sees the same product._

The key to feature targeting is allowing a feature (any behaviour of the system
which is either new or different from what the majority of users see) to be
turned on for a single user or any subset of users, selected according to
set criteria, or randomly (but in a "sticky" fashion).

This capability can then be used in place of a number of other commonly used
risk-reducing strategies for testing, deployment and release of software, e.g.
ephemeral and long-term environments, blue-green and canary deployments, etc.

The goal of feature targeting is to **fully separate deployments from releases**
and reduce complexity of tehnical infrastructure which is in place solely to
make sure customers don't see behaviours which are not ready for the public and
to reduce risk of issues caused by deployment.

### How it works

In order to target individual customers at any layer in the tech stack, the
expected state of each individual toggles needs to come from a single source
of truth unique to that customer. In web based systems this is a HTTP request
(or more generally an RPC call).

A good place to put the state of feature is therefore a request header. This can
be forwarded from service to service so that the feature state is the same through
the entire request tree serving a single customer request. In order to be safe,
the default state of a feature should be "disabled"

### Implicit and Explicit targeting

There are two ways to enable a feature for a particular user: the system
targeting them **implicitly** based on rules, or the user requesting the feature
**explicitly** without any configuration being in place.

The latter is useful in a development team, when people need to enable a
particular feature for themselves for development and testing purposes, or when
sharing work in progress with an individual or a small group of people.

The former is useful when introducing the feature gradually to a larger group
of users outside the team without speaking to them first, but also for enabling
features for a specific supported location, type of customer, language variation,
etc.

### Change lifecycle

Each change set made by an engineer should begin by introducing a new feature
flag. In its simplest form this is a conditional statement checking the presence
of a particular feature name on the incoming request.

The engineer can then make changes, deploy them to a shared environment and use
explicit targeting (say with a specific header) to enable the feature for
themselves, but nobody else. This can be repeated as many times as necessary.
An interesting side-effect is that the incremental code changes made are
immediately available to the wider team (known as "mainline development") reducing
queuing of dependent changes.

Once the feature has been mostly completed, a QA person can use explicit targeting
to do deeper exploratory testing and outside-in regression tests can be written
exercising the feature, also using explicit targeting.

Finally, when the trust in the feature has grown to a point of being ready for
release, implicit targeting can be used to enable the feature for the whole team,
all of staff, beta users, a percentage of the public or all of the public.

Once the feature is available for the full audience, the conditionals protecting
it can be removed, which concludes the lifecycle.

Notice that this could be done without excessive codebase branching and a large
number of replias of the live environment, but with the same level of safety.
This is the goal of this capability.

### General architecture

In order to process incoming user requests and inject the enabled features
into them based on explicit and implicit targeting rules in a technology
agnostic way, the simplest way is to introduce an ingress proxy, which can
consult a targeting service befor each user request.

> TODO diagram

The service uses the request and a set of rules as inputs (and possibly
consults more external services) to decide which features should be enabled
for this particular request. This is then injected by the proxy into the request
before it is forwarded to the first service in the chain/tree.

The services in the request path need to forward the enabled feature set to
their upstream services, so that the feautre set is consistent throughout.

This architecture strongly resembles the architecture of API gateway based
systems and service meshes, so where possible, it is better from a complexity
and performance perspective to plug into that existing infrastructure.

### Supported platforms

Currently the only supported platform is [Kubernetes](https://kubernetes.io/)
with [Istio](https://istio.io/) service mesh. The long-term goal is to support most
common platforms and a "platform-less" use-case. The main difference between them
is the injection point, the targeting service is common.
