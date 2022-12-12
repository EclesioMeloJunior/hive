## Raft Basics

#### Server states

- Leader
- Candidate
- Follower
  - issue no requests by their own, just respond requests from Leaders and Candidates
  - If a clients contacts a Follower, the follower redirects to the Leader

In normal operations, there is only one Leader and all the others nodes are followers!

The state flow is:

- When the client starts the initial state is: _Follower_
- as a _Follower_ if the client does not receive nothing from a _Leader_ for a certain period (a.k.a election timeout) of time we should start a new election then we should change our state to: _Candidate_
- as a _Candidate_ if the _election timeout_ keeps happening and no new _Leader_ is established we keep the state: _Candidate_
- as a _Candidate_ if we discover a new _Leader_ or a higher term we downgrade to: _Follower_
- as a _Candidate_ if we receive votes from majority of servers we upgrade to: _Leader_
- as a _Leader_ if we discover a server with a higher term we downgrade to: _Follower_

#### Terms

- Raft divide _time_ into terms of Abstract length
- They are numbered in sequential order and begins with an _election_
- When a Candidate becomes a leader then it servers as Leader for the rest of the _term_
- If no Candidate become a Leader then a new _term_ starts with a new election

##### General rules over terms

- Each server stores the _term_, which increases monotically over time and exchanges their terms whenever server comunicates.
- Rejects messages with out of date _terms_.

#### Leader Election

- Heartbeat
  - _Leaders_ should send AppendEntries Message without logs entries in order to keep their positions
  - If a server does not receive any heartbeat from the _Leader_ over a period of time (_election timeout_) then it must start a new election (new term) to choose a new _Leader_
