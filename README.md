# VRChat Moderation Reference Dashboard
A dashboard that moderators and mod-bots can run to keep tabs on the entire instance. It can emit Notices driven by configurable Advisories so that moderators are immediately aware of important things going on in an instance, such as a potential threat.

Current list of features:
* List of active users, sorted by top advisory severity and join time
* Icons for active advisories (an advisory applies to a user)
* Pop-up OS notifications for new notices (an advisory applies to a newly joined user or a newly switched avatar)
* Configurable advisories conditioned on names, trust ranks, account age, group membership, and some others

Planned features:
* "Networking": Send advisories and user lists to other mods over the network (usually from a mod-bot)
* Detect users blocking moderators: icon in user list & quick reference tab
* General-purpose log parser
* Use VRC audit logs to check if someone has been kicked recently
* User watchlists, for use in advisories

Tips:
* We suggest you use Scarlet in addition to VRCMRD, in particular for Scarlet's Discord integrations for moderation actions.
* You don't have to have a separate mod-bot running VRCMRD for it to work. However, it works better, especially to collaborate with other moderators, read moderation history, and reduce VRC server load if you use VRCMRD's networking features.

## Intended setup
The way it's supposed to work is you'll have a bot user on a dedicated device that:
* Has everyone's avatar **turned on** at least so that it downloads and renders the actual avatar
* Is persistently in the lobby somewhere, using a standard VRChat client configured for verbose logging
* Runs VRCMRD in host mode on the device, where it'll use its logs as a definitive source of knowledge
* Is a moderator of the group, so that it can determine when someone blocks the bot

and on most moderators' devices, they'll run VRCMRD in client mode which will let them:
* See the name and performance level of each avatar
* Identify every person in the lobby, including those who have left
* See the moderation history (as known to VRCMRD) of users in or recently in the lobby
* Log verbal warnings

TODO: document how to set it up and connect