## ADR 0004: Sync Model: Bare Git Repository as Rendezvous Point (No Master)
Context

Penna is local-first. Penna is multiplatform. Every device holds a full, independent git repository. Each of these repositories has complete history. The application needs a sync point between devices. No device or server can act as an authoritative "master" of the data. This kind of authority would break the local-first principle. This kind of authority would break the self-hostable principle.
Decision

Penna uses a bare git repository as a sync rendezvous point. The user self-hosts this bare repository. The user can host this repository on a home server, a NAS, or a small VPS. The user accesses this repository through SSH or a git daemon. This bare repository holds no application logic. This bare repository runs no Penna code. This bare repository performs no validation and no conflict resolution. This bare repository only stores commits, exactly like a normal git remote. This remote is equivalent to "origin". Devices push to this repository and pull from this repository. Devices perform these actions through normal git operations. These operations run through the git2 adapter.

Authority over the data is temporal. Authority over the data is not hierarchical. Whichever device resolves a conflict and pushes first becomes the latest agreed state. This state stays current only until another device pulls. This other device can then merge again. Non-conflicting changes merge silently through git's normal 3-way merge. These non-conflicting changes include different entries. These non-conflicting changes also include non-overlapping edits within the same entry. Silent merges need no user interaction.

True conflicts are always surfaced to the user. A true conflict happens when two devices edit the same line or the same block differently. The system surfaces these conflicts through the in-app merge resolution UI. ADR 0003 defines this UI. The system never resolves true conflicts automatically. The system never resolves true conflicts silently.

A device can stay offline for a long time. This device can diverge significantly from the remote. This situation is not a special case. This situation still produces a normal merge. This merge simply has a larger diff surface. The user must review this larger diff surface.
Alternatives Considered

    A central server as the source of truth, in a traditional client-server model — The team rejects this option. This model breaks the local-first principle. This model breaks the self-hostable principle. This model turns the server into a single point of failure. This model makes the server a mandatory dependency for the app to function.
    Automatic conflict resolution, through a "last write wins" heuristic or a similar heuristic — The team rejects this option. This method can silently destroy a user's journal content. This outcome is unacceptable for a personal journaling application.
    Peer-to-peer sync with no central rendezvous point at all — The team rejects this option for the first version. This method adds significant networking complexity. This complexity includes discovery. This complexity includes NAT traversal. This method brings no clear benefit over a simple self-hosted bare repository. A simple self-hosted bare repository already satisfies the self-hostable requirement.

Consequences

Positive:

    Users keep full control of their data and its location.
    The application needs no Penna server to run.
    The sync model reuses git's own well-tested merge and conflict machinery. The team does not build a custom sync protocol.
    The model works identically whether the user has two devices or ten devices.

Negative / Tradeoffs:

    The user must set up and maintain their own bare repository host. This step adds setup work compared to a ready-made cloud sync service.
    Very large or frequent divergence between devices can produce merge conflicts. The user must resolve these conflicts manually.
    The application must ship a capable in-app merge UI, as ADR 0003 describes. Without this UI, the model feels correct but not seamless.
