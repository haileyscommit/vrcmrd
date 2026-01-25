# VRChat Moderation Reference Dashboard
Gives each moderator of a group knowledge and understanding of all users in a lobby, including what avatars they're in and stuff. It's basically a re-implementation of Scarlet, but networked so you don't have to live-stream it & each mod gets more read-only access to stuff, and hopefully generally better.

The way it's supposed to work is you'll have a bot user on a dedicated device that:
* Has everyone's avatar **turned on** at least so that it downloads and renders the actual avatar
* Is persistently in the lobby somewhere, using a standard VRChat client configured for verbose logging
* Runs VRCMRD in host mode on the device, where it'll use its logs as a definitive source of knowledge
* Is a moderator of the group, so that it can determine when someone blocks the bot

and on most moderators' devices, they'll run VRCMRD in client mode which will let them:
* See the name and performance level of each avatar
* Identify every person in the lobby, including those
* See the moderation history (as known to VRCMRD) of users in or recently in the lobby
* Log verbal warnings
* Address moderation evidence tickets -- this _can_ be backed by Discord, which can allow some additional commentary, and access for moderators who cannot run VRCMRD (i.e. Quest standalone users).

Other niceties may include:
* World integrations
* Logging of instance events
* Graceful crash recovery when rejoining the same instance -- VRCMRD will determine the differences after the bot user rejoins
* Text-to-speech callouts and sound effects for users, group membership, avatars, bot blocking, or other instance events