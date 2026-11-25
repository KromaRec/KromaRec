# KromaRec 0.10.4 reference inventory

One row per pinned source registration, including repeated mounts. Implementation is the maintained code inventory. HTTP is one anonymous, empty-body local request on a fresh schema 9 database. A received HTTP response completes the transport probe. Contract PASS covers the specific asserted cases only. Other suites exercise additional routes without individual contract attribution here. Native acceptance is unverified throughout. Source lines, probe inputs, MIME types and exact error bodies are in the adjacent JSON.

| Service | Method and template | Implementation | HTTP | Contract test |
|---|---|---|---:|---|
| accounts | `PUT /account/me/profileimage` | MISSING | 404 | NOT TESTED |
| accounts | `PUT /account/me/bannerimage` | MISSING | 404 | NOT TESTED |
| accounts | `PUT /account/me/emoji` | MISSING | 404 | NOT TESTED |
| accounts | `PUT /account/me/bio` | IMPLEMENTED | 401 | NOT TESTED |
| accounts | `PUT /account/me/personalpronouns` | IMPLEMENTED | 401 | NOT TESTED |
| accounts | `PUT /account/me/identityflags` | MISSING | 404 | NOT TESTED |
| accounts | `POST /account/me/phone` | MISSING | 404 | NOT TESTED |
| accounts | `POST /account/me/email/confirm` | MISSING | 404 | NOT TESTED |
| accounts | `GET /account/me/email/pending` | MISSING | 404 | NOT TESTED |
| accounts | `POST /account/me/email` | MISSING | 404 | NOT TESTED |
| accounts | `PUT /account/me/username` | MISSING | 404 | NOT TESTED |
| accounts | `PUT /account/me/displayname` | MISSING | 404 | NOT TESTED |
| accounts | `GET /accountprivacysettings/:id{[0-9]+}` | MISSING | 404 | NOT TESTED |
| accounts | `GET /parentalcontrol/me` | MISSING | 404 | NOT TESTED |
| accounts | `POST /account/create` | IMPLEMENTED | 403 | NOT TESTED |
| accounts | `GET /account/:id` | IMPLEMENTED | 200 | NOT TESTED |
| accounts | `GET /account/:id/bio` | IMPLEMENTED | 200 | NOT TESTED |
| accounts | `POST /account/bulk` | PARTIALLY IMPLEMENTED | 200 | PASS |
| accounts | `GET /account/bulk` | PARTIALLY IMPLEMENTED | 200 | PASS |
| accounts | `GET /account/username/:name` | IMPLEMENTED | 404 | NOT TESTED |
| accounts | `GET /account/search` | MISSING | 404 | NOT TESTED |
| accounts | `GET /account/me` | IMPLEMENTED | 401 | PASS |
| accounts | `GET /emojiConfig/whitelistedEmojis` | IMPLEMENTED | 200 | NOT TESTED |
| accounts | `GET /` | IMPLEMENTED | 200 | PASS |
| accounts | `GET /openapi.json` | MISSING | 404 | NOT TESTED |
| ai | `POST /realtime-session/create` | MISSING | 404 | NOT TESTED |
| ai | `GET /makerai/user/balances` | MISSING | 404 | NOT TESTED |
| ai | `GET /makerai/user/access` | MISSING | 404 | NOT TESTED |
| ai | `GET /roomieai/user/facts` | MISSING | 404 | NOT TESTED |
| ai | `GET /roomieai/user/access` | MISSING | 404 | NOT TESTED |
| ai | `GET /gameai/room/:roomId{[0-9]+}/spendsummary` | MISSING | 404 | NOT TESTED |
| ai | `GET /gameai/user/access` | IMPLEMENTED | 401 | PASS |
| ai | `GET /` | IMPLEMENTED | 200 | PASS |
| ai | `GET /openapi.json` | MISSING | 404 | NOT TESTED |
| api | `MOUNT /` | NOT VERIFIED | — | NOT TESTED |
| api | `MOUNT /` | NOT VERIFIED | — | NOT TESTED |
| api | `MOUNT /` | NOT VERIFIED | — | NOT TESTED |
| api | `MOUNT /` | NOT VERIFIED | — | NOT TESTED |
| api | `MOUNT /` | NOT VERIFIED | — | NOT TESTED |
| api | `MOUNT /` | NOT VERIFIED | — | NOT TESTED |
| api | `MOUNT /` | NOT VERIFIED | — | NOT TESTED |
| api | `MOUNT /` | NOT VERIFIED | — | NOT TESTED |
| api | `MOUNT /` | NOT VERIFIED | — | NOT TESTED |
| api | `MOUNT /` | NOT VERIFIED | — | NOT TESTED |
| api | `MOUNT /` | NOT VERIFIED | — | NOT TESTED |
| api | `MOUNT /` | NOT VERIFIED | — | NOT TESTED |
| api | `GET /openapi.json` | MISSING | 404 | NOT TESTED |
| api | `GET /iam/me/channels/:type` | IMPLEMENTED | 200 | NOT TESTED |
| api | `POST /api/customAvatarItems/v1` | MISSING | 404 | NOT TESTED |
| api | `GET /api/customAvatarItems/v1/me` | MISSING | 404 | NOT TESTED |
| api | `POST /api/inventions/v2/delete` | MISSING | 404 | NOT TESTED |
| api | `POST /api/inventions/v2/unpublish` | MISSING | 404 | NOT TESTED |
| api | `POST /api/inventions/v4/publish` | MISSING | 404 | NOT TESTED |
| api | `POST /api/inventions/v2/update` | MISSING | 404 | NOT TESTED |
| api | `PUT /api/inventions/v2/metadata` | MISSING | 404 | NOT TESTED |
| api | `POST /api/inventions/v9/save` | MISSING | 404 | NOT TESTED |
| api | `POST /api/inventions/v6/save` | MISSING | 404 | NOT TESTED |
| api | `POST /api/inventions/v1/report` | MISSING | 404 | NOT TESTED |
| api | `POST /api/inventions/v2/cheer` | MISSING | 404 | NOT TESTED |
| api | `POST /api/inventions/v1/cheer` | MISSING | 404 | NOT TESTED |
| api | `POST /api/customAvatarItems/v1/:id{[0-9a-fA-F-]{36}}/report` | MISSING | 404 | NOT TESTED |
| api | `GET /api/inventions/v2/mine` | MISSING | 404 | NOT TESTED |
| api | `GET /api/inventions/v2/search` | MISSING | 404 | NOT TESTED |
| api | `GET /api/inventions/v1/fromcreators` | MISSING | 404 | NOT TESTED |
| api | `GET /api/inventions/v1/featureddormskins` | MISSING | 404 | NOT TESTED |
| api | `GET /api/inventions/v1/featured` | MISSING | 404 | NOT TESTED |
| api | `GET /api/inventions/v1/toptoday` | MISSING | 404 | NOT TESTED |
| api | `GET /api/inventions/v1/details` | MISSING | 404 | NOT TESTED |
| api | `POST /api/inventions/v1/settags` | MISSING | 404 | NOT TESTED |
| api | `POST /api/inventions/v1/updateprice` | MISSING | 404 | NOT TESTED |
| api | `GET /api/inventions/v3/publish` | MISSING | 404 | NOT TESTED |
| api | `GET /api/inventions/v1/update` | MISSING | 404 | NOT TESTED |
| api | `POST /api/inventions/v1/update` | MISSING | 404 | NOT TESTED |
| api | `GET /api/inventions/v1/version` | MISSING | 404 | NOT TESTED |
| api | `GET /api/inventions/v1/personaldetails/:inventionId{[0-9]+}` | MISSING | 404 | NOT TESTED |
| api | `GET /api/inventions/v1/room` | MISSING | 404 | NOT TESTED |
| api | `GET /api/inventions/v1/fulllineageowner` | MISSING | 404 | NOT TESTED |
| api | `POST /api/inventions/v2/batch` | MISSING | 404 | NOT TESTED |
| api | `GET /api/inventions/v2/batch` | MISSING | 404 | NOT TESTED |
| api | `GET /api/inventions/v1/tagfilters` | MISSING | 404 | NOT TESTED |
| api | `GET /api/inventions/v1` | MISSING | 404 | NOT TESTED |
| api | `GET /outfits/me/saved` | MISSING | 404 | NOT TESTED |
| api | `POST /outfits/bulk` | MISSING | 404 | NOT TESTED |
| api | `PUT /outfits/me` | MISSING | 404 | NOT TESTED |
| api | `GET /outfits/me` | MISSING | 404 | NOT TESTED |
| api | `POST /api/customAvatarItems/GetCustomAvatarItemCurrentSavesForLegacyAvatarItems` | MISSING | 404 | NOT TESTED |
| api | `GET /api/customAvatarItems/v2/fromCreator/:accountId{[0-9]+}` | MISSING | 404 | NOT TESTED |
| api | `POST /api/customAvatarItems/v1/bulk` | MISSING | 404 | NOT TESTED |
| api | `GET /api/customAvatarItems/v1/hot` | MISSING | 404 | NOT TESTED |
| api | `GET /api/customAvatarItems/v1/search` | MISSING | 404 | NOT TESTED |
| api | `GET /api/customAvatarItems/v1/featured` | MISSING | 404 | NOT TESTED |
| api | `DELETE /api/customAvatarItems/v1/:id{[0-9a-fA-F-]{36}}` | MISSING | 404 | NOT TESTED |
| api | `PUT /api/customAvatarItems/v1/:id{[0-9a-fA-F-]{36}}` | MISSING | 404 | NOT TESTED |
| api | `POST /api/customAvatarItems/v1` | MISSING | 404 | NOT TESTED |
| api | `GET /api/customAvatarItems/v1/minPriceForPublicItem` | MISSING | 404 | NOT TESTED |
| api | `GET /api/customAvatarItems/v1/isRenderingEnabled` | MISSING | 404 | NOT TESTED |
| api | `GET /api/customAvatarItems/v1/isCreationEnabled` | MISSING | 404 | NOT TESTED |
| api | `GET /api/customAvatarItems/v1/isCreationAllowedForAccount` | MISSING | 404 | NOT TESTED |
| api | `POST /api/avatar/v1/lockeditems/bulk` | MISSING | 404 | NOT TESTED |
| api | `POST /api/avatar/v2/gifts/generate` | MISSING | 404 | NOT TESTED |
| api | `GET /voice/config` | IMPLEMENTED | 200 | NOT TESTED |
| api | `POST /statsigUserProperties` | IMPLEMENTED | 200 | NOT TESTED |
| api | `GET /api/gameconfigs/v1/all` | IMPLEMENTED | 200 | NOT TESTED |
| api | `GET /api/versioncheck/islandedversions` | IMPLEMENTED | 200 | NOT TESTED |
| api | `GET /api/versioncheck/v4` | IMPLEMENTED | 200 | NOT TESTED |
| api | `GET /api/config/v2` | IMPLEMENTED | 200 | NOT TESTED |
| api | `GET /api/config/v1/backtrace` | IMPLEMENTED | 200 | NOT TESTED |
| api | `GET /api/config/v1/azurespeech` | IMPLEMENTED | 200 | NOT TESTED |
| api | `GET /api/config/v1/amplitude` | IMPLEMENTED | 200 | NOT TESTED |
| api | `GET /api/playerevents/v1/:eventId{[0-9]+}` | MISSING | 404 | NOT TESTED |
| api | `GET /api/playerevents/v1/:eventId{[0-9]+}/responses` | MISSING | 404 | NOT TESTED |
| api | `PUT /api/playerevents/v2/:eventId{[0-9]+}/name` | MISSING | 404 | NOT TESTED |
| api | `PUT /api/playerevents/v2/:eventId{[0-9]+}/description` | MISSING | 404 | NOT TESTED |
| api | `PUT /api/playerevents/v2/:eventId{[0-9]+}/tags` | MISSING | 404 | NOT TESTED |
| api | `PUT /api/playerevents/v2/:eventId{[0-9]+}/accessibility` | MISSING | 404 | NOT TESTED |
| api | `PUT /api/playerevents/v2/:eventId{[0-9]+}/time` | MISSING | 404 | NOT TESTED |
| api | `POST /api/playerevents/v2/:eventId{[0-9]+}` | MISSING | 404 | NOT TESTED |
| api | `GET /api/playerevents/v2/:eventId{[0-9]+}` | MISSING | 404 | NOT TESTED |
| api | `POST /api/playerevents/v2/delete/:eventId{[0-9]+}` | MISSING | 404 | NOT TESTED |
| api | `DELETE /api/playerevents/v2/delete/:eventId{[0-9]+}` | MISSING | 404 | NOT TESTED |
| api | `POST /api/playerevents/v2` | MISSING | 404 | NOT TESTED |
| api | `POST /api/playerevents/v1/bulkInvite` | MISSING | 404 | NOT TESTED |
| api | `POST /api/playerevents/v1/report` | MISSING | 404 | NOT TESTED |
| api | `POST /api/playerevents/v1/respond` | MISSING | 404 | NOT TESTED |
| api | `GET /api/playerevents/v1/bulk` | MISSING | 404 | NOT TESTED |
| api | `POST /api/playerevents/v1/bulk` | MISSING | 404 | NOT TESTED |
| api | `GET /api/playerevents/v1/search` | MISSING | 404 | NOT TESTED |
| api | `GET /api/playerevents/v1/searchlive` | MISSING | 404 | NOT TESTED |
| api | `GET /api/playerevents/v1/room/:roomId{[0-9]+}` | MISSING | 404 | NOT TESTED |
| api | `GET /api/playerevents/v1/club/:clubId{[0-9]+}` | MISSING | 404 | NOT TESTED |
| api | `GET /api/playerevents/v1/clubs` | MISSING | 404 | NOT TESTED |
| api | `GET /api/playerevents/v1/tagfilters` | MISSING | 404 | NOT TESTED |
| api | `GET /api/playerevents/v1/all/:playerId{[0-9]+}` | MISSING | 404 | NOT TESTED |
| api | `GET /api/playerevents/v1/all` | MISSING | 404 | NOT TESTED |
| api | `GET /api/playerevents/v1` | MISSING | 404 | NOT TESTED |
| api | `POST /api/gamesight/event` | MISSING | 404 | NOT TESTED |
| api | `GET /api/announcement/v1/get` | MISSING | 404 | NOT TESTED |
| api | `GET /api/CircuitChipLists/:list` | MISSING | 404 | NOT TESTED |
| api | `GET /api/communityboard/v2/current` | MISSING | 404 | NOT TESTED |
| api | `GET /api/keepsakes/categories` | MISSING | 404 | NOT TESTED |
| api | `GET /api/keepsakes/rooms/:roomId` | MISSING | 404 | NOT TESTED |
| api | `GET /api/keepsakes/globalconfig` | MISSING | 404 | NOT TESTED |
| api | `GET /api/activities/charades/v1/words/:activity` | MISSING | 404 | NOT TESTED |
| api | `POST /api/sanitize/v2/isAllowed/bulk` | MISSING | 404 | NOT TESTED |
| api | `POST /api/sanitize/v1/isPure` | MISSING | 404 | NOT TESTED |
| api | `POST /api/sanitize/v1` | MISSING | 404 | NOT TESTED |
| api | `PUT /api/players/v1/playerPhotoTaggingSetting` | MISSING | 404 | NOT TESTED |
| api | `GET /api/players/v1/playerPhotoTaggingSetting` | MISSING | 404 | NOT TESTED |
| api | `POST /api/images/v5/cheered/bulk` | MISSING | 404 | NOT TESTED |
| api | `GET /api/images/v5/cheered/bulk` | MISSING | 404 | NOT TESTED |
| api | `POST /api/images/v1/cheer` | MISSING | 404 | NOT TESTED |
| api | `POST /api/images/v1/:id{[0-9]+}/report` | MISSING | 404 | NOT TESTED |
| api | `GET /api/images/v6/:id{[0-9]+}` | MISSING | 404 | NOT TESTED |
| api | `GET /api/images/v6` | MISSING | 404 | NOT TESTED |
| api | `GET /api/images/v5/bulk` | MISSING | 404 | NOT TESTED |
| api | `GET /api/images/v1/slideshow` | MISSING | 404 | NOT TESTED |
| api | `GET /api/images/v3/feed/player/:playerId{[0-9]+}` | MISSING | 404 | NOT TESTED |
| api | `GET /api/images/v5/player/:playerId{[0-9]+}` | MISSING | 404 | NOT TESTED |
| api | `GET /api/images/v4/player/:playerId{[0-9]+}` | MISSING | 404 | NOT TESTED |
| api | `GET /api/images/v4/room/:roomId{[0-9]+}` | MISSING | 404 | NOT TESTED |
| api | `POST /api/images/v1/modifydescription` | MISSING | 404 | NOT TESTED |
| api | `POST /api/images/v2/modifyaccessibility` | MISSING | 404 | NOT TESTED |
| api | `POST /api/images/v1/deletesaved` | MISSING | 404 | NOT TESTED |
| api | `POST /api/images/v4/uploadsaved` | MISSING | 404 | NOT TESTED |
| api | `GET /api/images/v2/named` | MISSING | 404 | NOT TESTED |
| api | `GET /api/consumables/v2/getUnlocked` | MISSING | 404 | NOT TESTED |
| api | `GET /api/equipment/v2/getUnlocked` | MISSING | 404 | NOT TESTED |
| api | `POST /api/PlayerReporting/v1/deviceId` | MISSING | 404 | NOT TESTED |
| api | `POST /api/playerwarnings` | MISSING | 404 | NOT TESTED |
| api | `POST /api/PlayerReporting/v1/roomModKick` | MISSING | 404 | NOT TESTED |
| api | `POST /api/PlayerReporting/v1/instantKick` | MISSING | 404 | NOT TESTED |
| api | `POST /api/PlayerReporting/v3/voteToKick` | MISSING | 404 | NOT TESTED |
| api | `POST /api/chatreport/createChatReport` | MISSING | 404 | NOT TESTED |
| api | `POST /api/PlayerReporting/v3/create` | MISSING | 404 | NOT TESTED |
| api | `POST /api/PlayerReporting/v1/hile` | MISSING | 404 | NOT TESTED |
| api | `GET /api/referee/files` | MISSING | 404 | NOT TESTED |
| api | `POST /api/PlayerReporting/v1/referee` | MISSING | 404 | NOT TESTED |
| api | `GET /api/PlayerReporting/v1/voteToKickReasons` | MISSING | 404 | NOT TESTED |
| api | `GET /api/PlayerReporting/v1/moderationBlockDetails` | MISSING | 404 | NOT TESTED |
| api | `POST /api/PlayerReporting/v1/moderationBlockDetails` | MISSING | 404 | NOT TESTED |
| api | `GET /api/purchasableXpBoosts/activations` | MISSING | 404 | NOT TESTED |
| api | `GET /api/progressionEvents/record/:id{[0-9]+}` | MISSING | 404 | NOT TESTED |
| api | `GET /api/progressionEvents/event/:id{[0-9]+}` | MISSING | 404 | NOT TESTED |
| api | `GET /api/progressionEvents/active` | MISSING | 404 | NOT TESTED |
| api | `POST /api/v1/progression/bulk` | MISSING | 404 | NOT TESTED |
| api | `GET /api/players/v2/progression/bulk` | MISSING | 404 | NOT TESTED |
| api | `POST /api/players/v2/progression/bulk` | MISSING | 404 | NOT TESTED |
| api | `POST /api/players/v1/progression/bulk` | MISSING | 404 | NOT TESTED |
| api | `POST /api/PlayerCheer/v1/SetSelectedCheer` | MISSING | 404 | NOT TESTED |
| api | `POST /api/PlayerCheer/v1/create` | MISSING | 404 | NOT TESTED |
| api | `GET /api/playerReputation/v2/bulk` | MISSING | 404 | NOT TESTED |
| api | `POST /api/playerReputation/v2/bulk` | MISSING | 404 | NOT TESTED |
| api | `POST /api/playerReputation/v1/bulk` | MISSING | 404 | NOT TESTED |
| api | `GET /api/players/v1/progression/:id` | MISSING | 404 | NOT TESTED |
| api | `GET /api/players/v1/progression/xpEarnedToday` | MISSING | 404 | NOT TESTED |
| api | `GET /api/playerReputation/v1/:id` | MISSING | 404 | NOT TESTED |
| api | `POST /api/rooms/v1/verifyRole` | MISSING | 404 | NOT TESTED |
| api | `GET /api/rooms/v1/filters` | MISSING | 404 | NOT TESTED |
| api | `GET /api/quickPlay/v1/getandclear` | MISSING | 404 | NOT TESTED |
| api | `GET /api/roomkeys/v1/room` | MISSING | 404 | NOT TESTED |
| api | `GET /api/roomkeys/v1/mine` | MISSING | 404 | NOT TESTED |
| api | `GET /api/messages/v1/favoriteFriendOnlineStatus` | MISSING | 404 | NOT TESTED |
| api | `POST /api/messages/v1/friendOnlineStatus` | IMPLEMENTED | 401 | NOT TESTED |
| api | `POST /api/messages/v3/delete` | IMPLEMENTED | 401 | NOT TESTED |
| api | `GET /api/messages/v2/get` | IMPLEMENTED | 401 | NOT TESTED |
| api | `GET /api/relationships/v1/unfavorite` | IMPLEMENTED | 401 | NOT TESTED |
| api | `POST /api/relationships/v1/unfavorite` | IMPLEMENTED | 401 | NOT TESTED |
| api | `GET /api/relationships/v1/favorite` | IMPLEMENTED | 401 | NOT TESTED |
| api | `POST /api/relationships/v1/favorite` | IMPLEMENTED | 401 | NOT TESTED |
| api | `GET /api/relationships/v1/unmute` | IMPLEMENTED | 401 | NOT TESTED |
| api | `POST /api/relationships/v1/unmute` | IMPLEMENTED | 401 | NOT TESTED |
| api | `GET /api/relationships/v1/mute` | IMPLEMENTED | 401 | NOT TESTED |
| api | `POST /api/relationships/v1/mute` | IMPLEMENTED | 401 | NOT TESTED |
| api | `GET /api/relationships/v1/unignore` | IMPLEMENTED | 401 | NOT TESTED |
| api | `POST /api/relationships/v1/unignore` | IMPLEMENTED | 401 | NOT TESTED |
| api | `GET /api/relationships/v1/ignore` | IMPLEMENTED | 401 | NOT TESTED |
| api | `POST /api/relationships/v1/ignore` | IMPLEMENTED | 401 | NOT TESTED |
| api | `GET /api/relationships/v2/addfriend` | IMPLEMENTED | 401 | NOT TESTED |
| api | `POST /api/relationships/v2/addfriend` | IMPLEMENTED | 401 | NOT TESTED |
| api | `GET /api/relationships/v2/removefriend` | IMPLEMENTED | 401 | NOT TESTED |
| api | `POST /api/relationships/v2/removefriend` | IMPLEMENTED | 401 | NOT TESTED |
| api | `GET /api/relationships/v2/acceptfriendrequest` | IMPLEMENTED | 401 | NOT TESTED |
| api | `POST /api/relationships/v2/acceptfriendrequest` | IMPLEMENTED | 401 | NOT TESTED |
| api | `GET /api/relationships/v2/sendfriendrequest` | IMPLEMENTED | 401 | NOT TESTED |
| api | `POST /api/relationships/v2/sendfriendrequest` | IMPLEMENTED | 401 | NOT TESTED |
| api | `POST /api/messages/v1/sendMultiple` | MISSING | 404 | NOT TESTED |
| api | `POST /api/messages/v2/send` | MISSING | 404 | NOT TESTED |
| api | `GET /api/relationships/mutualfriends` | IMPLEMENTED | 401 | NOT TESTED |
| api | `GET /api/relationships/v2/get` | IMPLEMENTED | 401 | NOT TESTED |
| auth | `GET /oculus/nonce` | MISSING | 404 | NOT TESTED |
| auth | `GET /role/screenshare/:id` | MISSING | 404 | NOT TESTED |
| auth | `GET /role/moderator` | MISSING | 404 | NOT TESTED |
| auth | `GET /role/moderator/:id` | MISSING | 404 | NOT TESTED |
| auth | `GET /role/developer` | MISSING | 404 | NOT TESTED |
| auth | `GET /role/developer/:id` | MISSING | 404 | NOT TESTED |
| auth | `GET /privileges/me/restrictions` | MISSING | 404 | NOT TESTED |
| auth | `POST /account/resetpassword` | MISSING | 404 | NOT TESTED |
| auth | `POST /account/recoverpassword` | MISSING | 404 | NOT TESTED |
| auth | `POST /account/me/changepassword` | IMPLEMENTED | 401 | NOT TESTED |
| auth | `POST /connect/token` | PARTIALLY IMPLEMENTED | 415 | PASS |
| auth | `POST /connect/device/deny` | MISSING | 404 | NOT TESTED |
| auth | `POST /connect/device/approve` | MISSING | 404 | NOT TESTED |
| auth | `POST /connect/deviceauthorization` | MISSING | 404 | NOT TESTED |
| auth | `POST /cachedlogin/forplatformids` | MISSING | 404 | NOT TESTED |
| auth | `GET /cachedlogin/forplatformid/:platform/:id` | PARTIALLY IMPLEMENTED | 200 | NOT TESTED |
| auth | `POST /cachedlogin/forplatformid/:platform/:id` | PARTIALLY IMPLEMENTED | 200 | NOT TESTED |
| auth | `GET /eac/challenge` | MISSING | 404 | NOT TESTED |
| auth | `GET /openapi.json` | MISSING | 404 | NOT TESTED |
| cards | `GET /iam/me/channels/:channel` | IMPLEMENTED | 401 | NOT TESTED |
| cards | `GET /` | IMPLEMENTED | 200 | PASS |
| cdn | `GET /unityasset/:filename{.+}` | MISSING | 404 | NOT TESTED |
| cdn | `HEAD /unityasset/:filename{.+}` | MISSING | 404 | NOT TESTED |
| cdn | `GET /roomunityasset/:filename{.+}` | IMPLEMENTED | 404 | NOT TESTED |
| cdn | `GET /avatarunityasset/:filename{.+}` | MISSING | 404 | NOT TESTED |
| cdn | `GET /avatar/:asset{.+}` | IMPLEMENTED | 404 | NOT TESTED |
| cdn | `GET /video/:name{.+}` | MISSING | 404 | NOT TESTED |
| cdn | `GET /data/:id{.+}` | IMPLEMENTED | 404 | NOT TESTED |
| cdn | `GET /invention/:dataBlob{.+}` | IMPLEMENTED | 404 | NOT TESTED |
| cdn | `GET /room/:dataBlob{.+}` | IMPLEMENTED | 404 | NOT TESTED |
| cdn | `GET /sigs/:sigName` | IMPLEMENTED | 404 | NOT TESTED |
| cdn | `GET /config/:name` | IMPLEMENTED | 404 | NOT TESTED |
| cdn | `GET /config/LoadingScreenTipData` | IMPLEMENTED | 200 | NOT TESTED |
| cdn | `GET /` | IMPLEMENTED | 200 | PASS |
| cdn | `GET /openapi.json` | MISSING | 404 | NOT TESTED |
| chat | `GET /thread/:id{[0-9]+}/message` | MISSING | 404 | NOT TESTED |
| chat | `PUT /thread/:id{[0-9]+}/message/:messageId{[0-9]+}/read` | MISSING | 404 | NOT TESTED |
| chat | `POST /thread/:id{[0-9]+}/message/:messageId{[0-9]+}/read` | MISSING | 404 | NOT TESTED |
| chat | `PUT /thread/:id{[0-9]+}/read` | MISSING | 404 | NOT TESTED |
| chat | `POST /thread/:id{[0-9]+}/read` | MISSING | 404 | NOT TESTED |
| chat | `POST /thread/:id{[0-9]+}/member/:playerId{[0-9]+}` | MISSING | 404 | NOT TESTED |
| chat | `PUT /thread/:id{[0-9]+}/favorite` | MISSING | 404 | NOT TESTED |
| chat | `POST /thread/:id{[0-9]+}/favorite` | MISSING | 404 | NOT TESTED |
| chat | `POST /thread/:id{[0-9]+}/snooze` | MISSING | 404 | NOT TESTED |
| chat | `PUT /thread/:id{[0-9]+}/snooze` | MISSING | 404 | NOT TESTED |
| chat | `POST /thread/:id{[0-9]+}/leave` | MISSING | 404 | NOT TESTED |
| chat | `DELETE /thread/:id{[0-9]+}/leave` | MISSING | 404 | NOT TESTED |
| chat | `POST /thread/:id{[0-9]+}/rename` | MISSING | 404 | NOT TESTED |
| chat | `PUT /thread/:id{[0-9]+}/rename` | MISSING | 404 | NOT TESTED |
| chat | `POST /thread/:id{[0-9]+}/message` | MISSING | 404 | NOT TESTED |
| chat | `POST /thread/:id{[0-9]+}` | MISSING | 404 | NOT TESTED |
| chat | `GET /thread/:id{[0-9]+}` | MISSING | 404 | NOT TESTED |
| chat | `POST /thread/withmembers` | MISSING | 404 | NOT TESTED |
| chat | `GET /thread/checkCanSendDirectMessageWithPrivacySetting` | MISSING | 404 | NOT TESTED |
| chat | `PUT /thread/chatPrivacySetting` | MISSING | 404 | NOT TESTED |
| chat | `GET /thread/chatPrivacySetting` | MISSING | 404 | NOT TESTED |
| chat | `POST /thread/party` | MISSING | 404 | NOT TESTED |
| chat | `GET /thread/party` | MISSING | 404 | NOT TESTED |
| chat | `GET /settings/partyinvite` | MISSING | 404 | NOT TESTED |
| chat | `POST /thread` | MISSING | 404 | NOT TESTED |
| chat | `GET /thread` | MISSING | 404 | NOT TESTED |
| chat | `GET /` | IMPLEMENTED | 200 | PASS |
| chat | `GET /openapi.json` | MISSING | 404 | NOT TESTED |
| clubs | `POST /club/:clubId{[0-9]+}/leave` | MISSING | 404 | NOT TESTED |
| clubs | `POST /club/:clubId{[0-9]+}/join` | MISSING | 404 | NOT TESTED |
| clubs | `POST /club/:clubId{[0-9]+}/members/leave` | MISSING | 404 | NOT TESTED |
| clubs | `PUT /club/:clubId{[0-9]+}/members/invite` | MISSING | 404 | NOT TESTED |
| clubs | `PUT /club/:clubId{[0-9]+}/members/requesttojoin` | MISSING | 404 | NOT TESTED |
| clubs | `DELETE /club/:clubId{[0-9]+}` | MISSING | 404 | NOT TESTED |
| clubs | `GET /club/:clubId{[0-9]+}` | IMPLEMENTED | 404 | NOT TESTED |
| clubs | `PUT /club/:clubId{[0-9]+}/additionalimage/:index{[0-9]+}` | MISSING | 404 | NOT TESTED |
| clubs | `DELETE /club/:clubId{[0-9]+}/additionalimage/:index{[0-9]+}` | MISSING | 404 | NOT TESTED |
| clubs | `PUT /club/:clubId{[0-9]+}/mainimage` | MISSING | 404 | NOT TESTED |
| clubs | `GET /club/:clubId{[0-9]+}/mainimage` | MISSING | 404 | NOT TESTED |
| clubs | `PUT /club/:clubId{[0-9]+}/clubhouse` | MISSING | 404 | NOT TESTED |
| clubs | `DELETE /club/:clubId{[0-9]+}/clubhouse` | MISSING | 404 | NOT TESTED |
| clubs | `PUT /club/:clubId{[0-9]+}/minlevel` | MISSING | 404 | NOT TESTED |
| clubs | `GET /club/:clubId{[0-9]+}/members` | MISSING | 404 | NOT TESTED |
| clubs | `GET /club/:clubId{[0-9]+}/hasDisabledClubChat` | MISSING | 404 | NOT TESTED |
| clubs | `GET /club/:clubId{[0-9]+}/details` | MISSING | 404 | NOT TESTED |
| clubs | `PUT /club/:clubId{[0-9]+}/modifydetails` | MISSING | 404 | NOT TESTED |
| clubs | `PUT /club/:clubId{[0-9]+}/modify` | MISSING | 404 | NOT TESTED |
| clubs | `POST /club/create` | MISSING | 404 | NOT TESTED |
| clubs | `GET /club/categoryTags` | MISSING | 404 | NOT TESTED |
| clubs | `GET /club/search` | MISSING | 404 | NOT TESTED |
| clubs | `GET /club/account/:accountId{[0-9]+}/created` | MISSING | 404 | NOT TESTED |
| clubs | `GET /account/:accountId{[0-9]+}/clubs` | MISSING | 404 | NOT TESTED |
| clubs | `GET /club/mine/created` | MISSING | 404 | NOT TESTED |
| clubs | `GET /club/mine/member` | MISSING | 404 | NOT TESTED |
| clubs | `DELETE /announcements/club/:clubId{[0-9]+}/:announcementId{[0-9]+}` | MISSING | 404 | NOT TESTED |
| clubs | `POST /announcements/club/:clubId{[0-9]+}` | MISSING | 404 | NOT TESTED |
| clubs | `GET /announcements/club/:clubId{[0-9]+}` | MISSING | 404 | NOT TESTED |
| clubs | `GET /announcements/v2/subscription/mine/unread` | MISSING | 404 | NOT TESTED |
| clubs | `GET /announcements/v2/mine/unread` | MISSING | 404 | NOT TESTED |
| clubs | `GET /subscription/subscriberCount/:accountId{[0-9]+}` | MISSING | 404 | NOT TESTED |
| clubs | `POST /subscription/:accountId{[0-9]+}` | MISSING | 404 | NOT TESTED |
| clubs | `GET /subscription/details/:subscription` | MISSING | 404 | NOT TESTED |
| clubs | `GET /subscription/details/:accountId{[0-9]+}` | MISSING | 404 | NOT TESTED |
| clubs | `GET /subscription/mine/member` | MISSING | 404 | NOT TESTED |
| clubs | `DELETE /club/home/me` | MISSING | 404 | NOT TESTED |
| clubs | `PUT /club/home/me` | MISSING | 404 | NOT TESTED |
| clubs | `GET /club/home/me` | MISSING | 404 | NOT TESTED |
| clubs | `GET /openapi.json` | MISSING | 404 | NOT TESTED |
| commerce | `GET /reminder/currentTokenBundles/v2` | MISSING | 404 | NOT TESTED |
| commerce | `GET /purchasecampaign/allcurrent/v2` | MISSING | 404 | NOT TESTED |
| commerce | `GET /purchase/v1/cleanuppending` | MISSING | 404 | NOT TESTED |
| commerce | `POST /purchase/v1/cleanuppending` | MISSING | 404 | NOT TESTED |
| commerce | `GET /api/catalog/v1/all` | MISSING | 404 | NOT TESTED |
| commerce | `POST /purchase/v1/initiatepurchase` | MISSING | 404 | NOT TESTED |
| commerce | `GET /purchaseRestriction/isplayerrestricted` | MISSING | 404 | NOT TESTED |
| commerce | `GET /purchase/v1/hasspentmoney` | IMPLEMENTED | 200 | NOT TESTED |
| commerce | `GET /` | IMPLEMENTED | 200 | PASS |
| commerce | `GET /openapi.json` | MISSING | 404 | NOT TESTED |
| datacollection | `GET /sampling` | INTENTIONALLY UNSUPPORTED | — | NOT TESTED |
| datacollection | `POST /data/events` | INTENTIONALLY UNSUPPORTED | — | NOT TESTED |
| datacollection | `POST /data/event` | INTENTIONALLY UNSUPPORTED | — | NOT TESTED |
| datacollection | `GET /` | INTENTIONALLY UNSUPPORTED | — | NOT TESTED |
| discovery | `GET /sections/pagesource/:type` | MISSING | 404 | NOT TESTED |
| discovery | `GET /sections/bulk` | IMPLEMENTED | 200 | NOT TESTED |
| discovery | `GET /` | IMPLEMENTED | 200 | PASS |
| discovery | `GET /openapi.json` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/influencerpartnerprogram/remove` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/influencerpartnerprogram/support` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/influencerpartnerprogram/myinfluencer` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/influencerpartnerprogram/influencer` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/influencerpartnerprogram/influencers` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/incentivizedreferrals/progress` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/makerai/checkfreetrialeligibility` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/subscriptionseasons/v1/seasons/current` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/CampusCard/v1/UpdateAndGetSubscription` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/CampusCard/v1/SignUpBonus` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/roomkeys/v1/owns/bulk` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/roomkeys/v1/awardbulk` | MISSING | 404 | NOT TESTED |
| econ | `PUT /api/roomkeys/v1/updateAll` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/storefronts/v1/PurchaseRoomKeyWithCurrency` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/storefronts/v1/buyRoomKey` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/roomkeys/v1/create` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/roomkeys/v1/room` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/roomkeys/v1/mine` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/gamerewards/v1/request` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/gamerewards/v1/pending` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/challenge/v2/updateProgress` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/challenge/v2/getCurrent` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/storefronts/v1/adcarouselitems` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/storefronts/v3/buyInvention` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/storefronts/v2/buyInvention` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/items/bulkpurchase` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/storefronts/v2/buyItem` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/storefronts/v3/giftdropstore/:id` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/storefronts/v4/balance/:currencyType` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/consumables/v1/updateActive` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/consumables/v1/consume` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/consumables/v2/getUnlocked` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/consumables/v1/query/bulk` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/consumables/v1/all` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/storefronts/v1/purchasableitems` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/items/purchaseInfos` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/ugcPurchasables/v1/items/bulk` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/ugcPurchasables/v1/items/room/:roomId{[0-9]+}` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/distanceMovedRewardTracks/distance` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/timedCurrencyGrants/update` | MISSING | 404 | NOT TESTED |
| econ | `GET /paintball-battle-pass/config` | MISSING | 404 | NOT TESTED |
| econ | `GET /econ/roomEconConfig/:roomId` | MISSING | 404 | NOT TESTED |
| econ | `GET /econ/roomGiftDropShops/room/:roomId` | MISSING | 404 | NOT TESTED |
| econ | `GET /econ/roomOffer/room/:roomId/recentPurchases` | MISSING | 404 | NOT TESTED |
| econ | `GET /econ/roomOffer/room/:roomId/purchaseCounts` | MISSING | 404 | NOT TESTED |
| econ | `GET /econ/roomOffer/room/:roomId` | MISSING | 404 | NOT TESTED |
| econ | `GET /econ/roomInventoryItemTags/room/:roomId` | MISSING | 404 | NOT TESTED |
| econ | `GET /econ/roomInventory/room/:roomId/player` | MISSING | 404 | NOT TESTED |
| econ | `GET /econ/roomInventory/room/:roomId` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/roomCurrencies/v2/purchase` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/roomcurrencies/v1/getBalance` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/roomcurrencies/v1/getAllBalances` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/roomcurrencies/v1/createPurchaseOffer` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/roomcurrencies/v1/getPurchaseOffersBatch` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/roomcurrencies/v1/awardCurrency/bulk` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/roomcurrencies/v1/deletePurchaseOffer` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/roomcurrencies/v1/createPurchaseOffer` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/roomcurrencies/v1/getPurchaseOffersBatch` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/roomcurrencies/v1/updateCurrency` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/roomcurrencies/v1/createCurrency` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/roomcurrencies/v1/currencies` | MISSING | 404 | NOT TESTED |
| econ | `PUT /api/roomconsumables/v1/roomConsumable/:id/consume` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/roomconsumables/v1/roomConsumable/room/:roomId/me` | MISSING | 404 | NOT TESTED |
| econ | `PUT /api/roomconsumables/v1/roomConsumable` | MISSING | 404 | NOT TESTED |
| econ | `PUT /api/roomconsumables/v1/roomconsumable/:id/purchase/:via{currency\|tokens}` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/roomconsumables/v1/roomConsumable/awardBulk` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/roomconsumables/v1/roomConsumable/room/:roomId` | MISSING | 404 | NOT TESTED |
| econ | `PUT /api/equipment/v1/update` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/equipment/v1/update` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/equipment/v2/getUnlocked` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/avatar/v2/:id` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/avatar/v2/gifts/consume` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/avatar/v2/gifts` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/avatar/v4/saved/set` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/avatar/v3/saved/set` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/avatar/v3/saved` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/itemWishlists/v1/isonwishlist/bulk` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/itemWishlists/v1/wishlist/remove` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/itemWishlists/v1/wishlist/add` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/itemWishlists/v1/wishlist/:accountId{[0-9]+}` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/itemWishlists/v1/wishlist/me` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/checklist/v1/complete` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/checklist/v2/complete` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/checklist/v1/current` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/checklist/v2/current` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/avatar/v2/set` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/avatar/v2` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/objectives/v1/updateobjective` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/objectives/v1/cleargroup` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/objectives/v1/cleargroup` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/objectives/v1/myprogress` | MISSING | 404 | NOT TESTED |
| econ | `GET /econ/customAvatarItems/v1/owned` | MISSING | 404 | NOT TESTED |
| econ | `POST /circuitsRoomRewards/grantRoomBadge` | MISSING | 404 | NOT TESTED |
| econ | `GET /econ/customAvatarItems/v1/ownedby/:accountId{[0-9]+}` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/avatar/v4/items` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/avatar/v1/defaultbaseavataritems` | MISSING | 404 | NOT TESTED |
| econ | `GET /api/avatar/v1/defaultunlocked` | MISSING | 404 | NOT TESTED |
| econ | `POST /api/avatar/v1/lockeditems/bulk` | MISSING | 404 | NOT TESTED |
| econ | `GET /openapi.json` | MISSING | 404 | NOT TESTED |
| img | `GET /` | IMPLEMENTED | 200 | PASS |
| img | `GET /openapi.json` | MISSING | 404 | NOT TESTED |
| img | `GET /:key{.+}` | MISSING | 404 | NOT TESTED |
| leaderboard | `POST /leaderboard/CheckAndSetStat` | MISSING | 404 | NOT TESTED |
| leaderboard | `POST /leaderboard/GetPlayerRank` | MISSING | 404 | NOT TESTED |
| leaderboard | `POST /leaderboard/GetRanks` | MISSING | 404 | NOT TESTED |
| leaderboard | `POST /leaderboard/GetNearbyScores` | MISSING | 404 | NOT TESTED |
| leaderboard | `GET /` | IMPLEMENTED | 200 | PASS |
| leaderboard | `GET /openapi.json` | MISSING | 404 | NOT TESTED |
| link | `POST /actionlink` | MISSING | 404 | NOT TESTED |
| link | `GET /` | IMPLEMENTED | 200 | PASS |
| lists | `POST /contextualfeatures` | MISSING | 404 | NOT TESTED |
| lists | `GET /algorithmiclists/:list` | MISSING | 404 | NOT TESTED |
| lists | `DELETE /curatedlists/:listId/items/:itemId` | MISSING | 404 | NOT TESTED |
| lists | `PUT /curatedlists/:name/items/:itemId/createlistifneeded` | MISSING | 404 | NOT TESTED |
| lists | `GET /curatedlists` | IMPLEMENTED | 200 | NOT TESTED |
| lists | `GET /curatedlists/bulk` | MISSING | 404 | NOT TESTED |
| lists | `GET /` | IMPLEMENTED | 200 | PASS |
| lists | `GET /openapi.json` | MISSING | 404 | NOT TESTED |
| match | `GET /rooms/requiring/rrplus` | MISSING | 401 | NOT TESTED |
| match | `GET /rooms/requiring/developer` | MISSING | 401 | NOT TESTED |
| match | `GET /room/:roomId{[0-9]+}/instances` | IMPLEMENTED | 401 | NOT TESTED |
| match | `POST /room/:roomId{[0-9]+}/instances/clear` | MISSING | 401 | NOT TESTED |
| match | `POST /roominstance/:id/markprivate` | IMPLEMENTED | 401 | NOT TESTED |
| match | `PUT /roominstance/:id/matchpolicy` | IMPLEMENTED | 401 | NOT TESTED |
| match | `PUT /roominstance/:id/inprogress` | PARTIALLY IMPLEMENTED | 401 | NOT TESTED |
| match | `POST /roominstance/:id/reportjoinresult` | PARTIALLY IMPLEMENTED | 401 | NOT TESTED |
| match | `POST /invite` | PARTIALLY IMPLEMENTED | 401 | NOT TESTED |
| match | `PUT /player/gameserverregionpings` | IMPLEMENTED | 200 | PASS |
| match | `PUT /player/photonregionpings` | IMPLEMENTED | 200 | PASS |
| match | `GET /player/qos` | MISSING | 401 | NOT TESTED |
| match | `GET /player/connection-info` | PARTIALLY IMPLEMENTED | 401 | PASS |
| match | `POST /matchmake/dorm` | PARTIALLY IMPLEMENTED | 401 | NOT TESTED |
| match | `POST /matchmake/none` | PARTIALLY IMPLEMENTED | 401 | NOT TESTED |
| match | `POST /matchmake/v2/room/:roomId` | MISSING | 401 | NOT TESTED |
| match | `POST /matchmake/v2/room/:roomId/:subRoomId{[0-9]+}` | MISSING | 401 | NOT TESTED |
| match | `POST /matchmake/room/:roomId` | PARTIALLY IMPLEMENTED | 401 | NOT TESTED |
| match | `POST /matchmake/room/:roomId/:subRoomId{[0-9]+}` | PARTIALLY IMPLEMENTED | 401 | NOT TESTED |
| match | `POST /matchmake/instance/:instanceId{[0-9]+}` | PARTIALLY IMPLEMENTED | 401 | NOT TESTED |
| match | `POST /matchmake/invite/:inviteId{[0-9]+}` | MISSING | 401 | NOT TESTED |
| match | `POST /matchmake/v2/player/:playerId{[0-9]+}` | MISSING | 401 | NOT TESTED |
| match | `POST /matchmake/player/:playerId{[0-9]+}` | PARTIALLY IMPLEMENTED | 401 | NOT TESTED |
| match | `POST /matchmake/event/:eventId{[0-9]+}` | MISSING | 401 | NOT TESTED |
| match | `POST /matchmake/club/:clubId{[0-9]+}` | MISSING | 401 | NOT TESTED |
| match | `PUT /player/avoidjuniors` | MISSING | 401 | NOT TESTED |
| match | `GET /player/avoidjuniors` | MISSING | 401 | NOT TESTED |
| match | `PUT /player/vrmovementmode` | MISSING | 401 | NOT TESTED |
| match | `PUT /player/statusvisibility` | MISSING | 401 | NOT TESTED |
| match | `POST /player/heartbeat` | IMPLEMENTED | 401 | NOT TESTED |
| match | `POST /player` | PARTIALLY IMPLEMENTED | 401 | NOT TESTED |
| match | `GET /player` | PARTIALLY IMPLEMENTED | 401 | NOT TESTED |
| match | `GET /clubhousesearch/mostactivenow` | MISSING | 401 | NOT TESTED |
| match | `GET /tachyon` | MISSING | 401 | NOT TESTED |
| match | `POST /player/notifydisconnect` | MISSING | 401 | NOT TESTED |
| match | `POST /player/logout` | IMPLEMENTED | 200 | NOT TESTED |
| match | `POST /player/verifylock` | PARTIALLY IMPLEMENTED | 401 | NOT TESTED |
| match | `POST /player/exclusivelogin` | PARTIALLY IMPLEMENTED | 401 | NOT TESTED |
| match | `POST /player/login` | IMPLEMENTED | 200 | NOT TESTED |
| match | `GET /openapi.json` | MISSING | 401 | NOT TESTED |
| moderation | `GET /voice/config` | IMPLEMENTED | 200 | NOT TESTED |
| moderation | `GET /` | IMPLEMENTED | 200 | PASS |
| notify | `DELETE /internal/hub-state/pending` | MISSING | 404 | NOT TESTED |
| notify | `GET /internal/hub-state` | MISSING | 404 | NOT TESTED |
| notify | `POST /internal/coach-message` | MISSING | 404 | NOT TESTED |
| notify | `POST /internal/coach-message-all` | MISSING | 404 | NOT TESTED |
| notify | `POST /internal/broadcast` | MISSING | 404 | NOT TESTED |
| notify | `POST /internal/notify` | MISSING | 404 | NOT TESTED |
| notify | `GET /hub/v1` | PARTIALLY IMPLEMENTED | 401 | NOT TESTED |
| notify | `POST /hub/v1/negotiate` | IMPLEMENTED | 200 | NOT TESTED |
| ns | `GET /` | IMPLEMENTED | 200 | PASS |
| platformnotifications | `GET /preferences` | MISSING | 404 | NOT TESTED |
| platformnotifications | `GET /crm/me/config/v3` | MISSING | 404 | NOT TESTED |
| platformnotifications | `GET /config/categories` | MISSING | 404 | NOT TESTED |
| platformnotifications | `GET /accounts/:id{[0-9]+}/receives/GameplayInvites` | MISSING | 404 | NOT TESTED |
| platformnotifications | `GET /` | IMPLEMENTED | 200 | PASS |
| playersettings | `DELETE /playersettings` | IMPLEMENTED | 401 | NOT TESTED |
| playersettings | `PUT /playersettings` | IMPLEMENTED | 401 | NOT TESTED |
| playersettings | `GET /playersettings` | IMPLEMENTED | 401 | PASS |
| playersettings | `GET /` | IMPLEMENTED | 200 | PASS |
| playersettings | `GET /openapi.json` | MISSING | 404 | NOT TESTED |
| roomcomments | `POST /comments/create/:roomId` | MISSING | 404 | NOT TESTED |
| roomcomments | `GET /comments/get/:roomId` | IMPLEMENTED | 200 | NOT TESTED |
| roomcomments | `GET /` | IMPLEMENTED | 200 | PASS |
| roomcomments | `GET /openapi.json` | MISSING | 404 | NOT TESTED |
| rooms | `GET /photon_access_token` | PARTIALLY IMPLEMENTED | 401 | NOT TESTED |
| rooms | `POST /unity_assets/baked/bulk` | MISSING | 401 | NOT TESTED |
| rooms | `GET /publishState/configs` | MISSING | 404 | NOT TESTED |
| rooms | `GET /rooms/:roomId{[0-9]+}` | PARTIALLY IMPLEMENTED | 404 | NOT TESTED |
| rooms | `POST /rooms/:roomId{[0-9]+}/experience/player` | MISSING | 401 | NOT TESTED |
| rooms | `GET /rooms/:roomId{[0-9]+}/experience/player` | MISSING | 404 | NOT TESTED |
| rooms | `POST /rooms/:roomId{[0-9]+}/experience` | MISSING | 401 | NOT TESTED |
| rooms | `GET /rooms/:roomId{[0-9]+}/experience` | MISSING | 404 | NOT TESTED |
| rooms | `PUT /rooms/:roomId{[0-9]+}/playerdata/me` | MISSING | 401 | NOT TESTED |
| rooms | `GET /rooms/:roomId{[0-9]+}/playerdata/me` | MISSING | 404 | NOT TESTED |
| rooms | `GET /rooms/:roomId{[0-9]+}/similar` | MISSING | 404 | NOT TESTED |
| rooms | `DELETE /rooms/:roomId{[0-9]+}/subrooms/:subRoomId{[0-9]+}` | MISSING | 401 | NOT TESTED |
| rooms | `POST /rooms/:roomId{[0-9]+}/subrooms` | MISSING | 401 | NOT TESTED |
| rooms | `POST /rooms/:roomId{[0-9]+}/subrooms/:subRoomId{[0-9]+}/move` | MISSING | 401 | NOT TESTED |
| rooms | `POST /rooms/:roomId{[0-9]+}/subrooms/:subRoomId{[0-9]+}/clone` | MISSING | 401 | NOT TESTED |
| rooms | `PUT /rooms/:roomId{[0-9]+}/subrooms/:subRoomId{[0-9]+}/permissions` | MISSING | 401 | NOT TESTED |
| rooms | `PUT /rooms/:roomId{[0-9]+}/subrooms/:subRoomId{[0-9]+}/accessibility` | MISSING | 401 | NOT TESTED |
| rooms | `POST /rooms/:roomId{[0-9]+}/subrooms/:subRoomId{[0-9]+}/publish_save` | MISSING | 401 | NOT TESTED |
| rooms | `PUT /rooms/:roomId{[0-9]+}/subrooms/:subRoomId{[0-9]+}/modify` | MISSING | 401 | NOT TESTED |
| rooms | `POST /rooms/:roomId{[0-9]+}/subrooms/:subRoomId{[0-9]+}/data` | MISSING | 401 | NOT TESTED |
| rooms | `GET /rooms/:roomId{[0-9]+}/subrooms/:subRoomId{[0-9]+}/unityasset` | MISSING | 404 | NOT TESTED |
| rooms | `GET /rooms/:roomId{[0-9]+}/subrooms/:subRoomId{[0-9]+}/saves/:saveId{[0-9]+}` | MISSING | 404 | NOT TESTED |
| rooms | `GET /rooms/:roomId{[0-9]+}/subrooms/:subRoomId{[0-9]+}/saves/no_unity_assets` | MISSING | 404 | NOT TESTED |
| rooms | `GET /rooms/:roomId{[0-9]+}/subrooms/:subRoomId{[0-9]+}/saves` | MISSING | 404 | NOT TESTED |
| rooms | `PUT /rooms/:roomId{[0-9]+}/exclude` | MISSING | 401 | NOT TESTED |
| rooms | `PUT /rooms/:roomId{[0-9]+}/accessibility` | PARTIALLY IMPLEMENTED | 401 | NOT TESTED |
| rooms | `PUT /rooms/:roomId{[0-9]+}/loadscreen` | MISSING | 401 | NOT TESTED |
| rooms | `PUT /rooms/:roomId{[0-9]+}/restrictions` | MISSING | 401 | NOT TESTED |
| rooms | `PUT /rooms/:roomId{[0-9]+}/v2/restrictions` | MISSING | 401 | NOT TESTED |
| rooms | `PUT /rooms/:roomId{[0-9]+}/max_player_calculation_mode` | MISSING | 401 | NOT TESTED |
| rooms | `PUT /rooms/:roomId{[0-9]+}/cloning` | MISSING | 401 | NOT TESTED |
| rooms | `PUT /rooms/:roomId{[0-9]+}/warning` | MISSING | 401 | NOT TESTED |
| rooms | `DELETE /rooms/:roomId{[0-9]+}/leaderboards/:leaderboardId{[0-9]+}` | MISSING | 401 | NOT TESTED |
| rooms | `POST /rooms/:roomId{[0-9]+}/leaderboards/:leaderboardId{[0-9]+}` | MISSING | 401 | NOT TESTED |
| rooms | `DELETE /rooms/:roomId{[0-9]+}/bans/:playerId{[0-9]+}` | MISSING | 401 | NOT TESTED |
| rooms | `DELETE /rooms/:roomId{[0-9]+}/bans/bulk` | MISSING | 401 | NOT TESTED |
| rooms | `GET /rooms/:roomId{[0-9]+}/bans/:playerId{[0-9]+}/isBanned` | MISSING | 404 | NOT TESTED |
| rooms | `GET /Room_server/rooms/:roomId{[0-9]+}/bans/:playerId{[0-9]+}/isBanned` | MISSING | 404 | NOT TESTED |
| rooms | `POST /rooms/:roomId{[0-9]+}/bans` | MISSING | 401 | NOT TESTED |
| rooms | `GET /rooms/:roomId{[0-9]+}/bans/history` | MISSING | 404 | NOT TESTED |
| rooms | `GET /rooms/:roomId{[0-9]+}/bans` | MISSING | 404 | NOT TESTED |
| rooms | `PUT /rooms/:roomId{[0-9]+}/roles/:accountId{[0-9]+}/invite` | PARTIALLY IMPLEMENTED | 401 | NOT TESTED |
| rooms | `PUT /rooms/:roomId{[0-9]+}/creator` | PARTIALLY IMPLEMENTED | 401 | NOT TESTED |
| rooms | `PUT /rooms/:roomId{[0-9]+}/roles/:accountId{[0-9]+}` | PARTIALLY IMPLEMENTED | 401 | NOT TESTED |
| rooms | `GET /rooms/:roomId{[0-9]+}/roles` | PARTIALLY IMPLEMENTED | 200 | NOT TESTED |
| rooms | `DELETE /rooms/:roomId{[0-9]+}` | PARTIALLY IMPLEMENTED | 401 | NOT TESTED |
| rooms | `PUT /rooms/:roomId{[0-9]+}/image` | MISSING | 401 | NOT TESTED |
| rooms | `PUT /rooms/:roomId{[0-9]+}/tags` | MISSING | 401 | NOT TESTED |
| rooms | `PUT /rooms/:roomId{[0-9]+}/name` | PARTIALLY IMPLEMENTED | 401 | NOT TESTED |
| rooms | `PUT /rooms/:roomId{[0-9]+}/description` | PARTIALLY IMPLEMENTED | 401 | NOT TESTED |
| rooms | `POST /rooms/:roomId{[0-9]+}/clone` | PARTIALLY IMPLEMENTED | 401 | NOT TESTED |
| rooms | `DELETE /rooms/:roomId{[0-9]+}/interactionby/me/favorite` | MISSING | 401 | NOT TESTED |
| rooms | `PUT /rooms/:roomId{[0-9]+}/interactionby/me/favorite` | MISSING | 401 | NOT TESTED |
| rooms | `DELETE /rooms/:roomId{[0-9]+}/interactionby/me/cheer` | MISSING | 401 | NOT TESTED |
| rooms | `PUT /rooms/:roomId{[0-9]+}/interactionby/me/cheer` | MISSING | 401 | NOT TESTED |
| rooms | `GET /rooms/:roomId{[0-9]+}/interactionby/me` | MISSING | 404 | NOT TESTED |
| rooms | `GET /rooms/:roomId{[0-9]+}/boosts/:playerId{[0-9]+}/count` | MISSING | 404 | NOT TESTED |
| rooms | `GET /rooms/visitedby/:playerId{[0-9]+}` | MISSING | 404 | NOT TESTED |
| rooms | `GET /rooms/visitedby/me/count` | MISSING | 404 | NOT TESTED |
| rooms | `GET /rooms/visitedby/me` | MISSING | 404 | NOT TESTED |
| rooms | `GET /rooms/favoritedby/me` | MISSING | 404 | NOT TESTED |
| rooms | `GET /showcase/:playerId{[0-9]+}` | MISSING | 404 | NOT TESTED |
| rooms | `GET /rooms/ownedby/:accountId{[0-9]+}` | MISSING | 404 | NOT TESTED |
| rooms | `PUT /dormroom/me` | MISSING | 401 | NOT TESTED |
| rooms | `GET /dormroom/me` | IMPLEMENTED | 401 | NOT TESTED |
| rooms | `GET /rooms/contributedby/me` | IMPLEMENTED | 401 | NOT TESTED |
| rooms | `GET /rooms/createdby/me` | IMPLEMENTED | 401 | NOT TESTED |
| rooms | `GET /rooms/ownedby/me` | IMPLEMENTED | 401 | NOT TESTED |
| rooms | `GET /roomserver/rooms/createdby/me` | IMPLEMENTED | 401 | NOT TESTED |
| rooms | `POST /rooms/bulk` | PARTIALLY IMPLEMENTED | 200 | NOT TESTED |
| rooms | `GET /rooms/bulk` | PARTIALLY IMPLEMENTED | 400 | NOT TESTED |
| rooms | `GET /featuredrooms/current` | MISSING | 404 | NOT TESTED |
| rooms | `GET /rooms/recommendations` | MISSING | 404 | NOT TESTED |
| rooms | `GET /rooms/base` | PARTIALLY IMPLEMENTED | 200 | NOT TESTED |
| rooms | `GET /rooms/carousel/rising` | MISSING | 404 | NOT TESTED |
| rooms | `GET /rooms/curated_playlists` | MISSING | 404 | NOT TESTED |
| rooms | `GET /rooms/hot` | MISSING | 404 | NOT TESTED |
| rooms | `GET /rooms/autocomplete_search` | MISSING | 404 | NOT TESTED |
| rooms | `GET /rooms/search/name` | IMPLEMENTED | 200 | NOT TESTED |
| rooms | `GET /rooms/search` | PARTIALLY IMPLEMENTED | 200 | NOT TESTED |
| rooms | `POST /rooms/create` | PARTIALLY IMPLEMENTED | 401 | NOT TESTED |
| rooms | `GET /rooms` | PARTIALLY IMPLEMENTED | 400 | NOT TESTED |
| rooms | `GET /` | IMPLEMENTED | 200 | PASS |
| rooms | `GET /openapi.json` | MISSING | 404 | NOT TESTED |
| storage | `POST /upload` | MISSING | 404 | NOT TESTED |
| storage | `POST /upload/video` | MISSING | 404 | NOT TESTED |
| storage | `POST /upload/unityassets` | MISSING | 404 | NOT TESTED |
| storage | `GET /` | IMPLEMENTED | 200 | PASS |
| storage | `GET /openapi.json` | MISSING | 404 | NOT TESTED |
| studio | `POST /internal/cloud-builds/:id/finish` | MISSING | 404 | NOT TESTED |
| studio | `POST /internal/cloud-builds/:id/heartbeat` | MISSING | 404 | NOT TESTED |
| studio | `POST /internal/cloud-builds/claim` | MISSING | 404 | NOT TESTED |
| studio | `POST /cloud-builds/cancel` | MISSING | 404 | NOT TESTED |
| studio | `PUT /cloud-builds/cancel` | MISSING | 404 | NOT TESTED |
| studio | `GET /cloud-builds` | IMPLEMENTED | 400 | NOT TESTED |
| studio | `POST /cloud-builds/create` | MISSING | 404 | NOT TESTED |
| studio | `POST /cloud-builds/custom-avatar-item/create-for-new-build-version` | MISSING | 404 | NOT TESTED |
| studio | `POST /cloud-builds/custom-avatar-item/:itemId{[0-9a-fA-F-]{36}}/save/:saveId{[1-9][0-9]*}` | MISSING | 404 | NOT TESTED |
| studio | `GET /cloud-builds/custom-avatar-item/:itemId{[0-9a-fA-F-]{36}}` | MISSING | 404 | NOT TESTED |
| studio | `GET /1/apps/:secret` | MISSING | 404 | NOT TESTED |
| studio | `GET /1/apps/:secret/*` | MISSING | 404 | NOT TESTED |
| studio | `GET /collaboration/owners-in-room` | MISSING | 404 | NOT TESTED |
| studio | `GET /cloud-builds/get-build-count-for-user` | MISSING | 404 | NOT TESTED |
| studio | `GET /cloud-builds/get-rate-limit-config` | MISSING | 404 | NOT TESTED |
| studio | `POST /cloud-builds/from-editor` | MISSING | 404 | NOT TESTED |
| studio | `GET /cloud-builds/for-room` | MISSING | 404 | NOT TESTED |
| studio | `GET /commonQuery/rrstudio/chiprecommendations` | MISSING | 404 | NOT TESTED |
| studio | `GET /access/override/:accountId{[1-9][0-9]*}` | MISSING | 404 | NOT TESTED |
| studio | `GET /` | IMPLEMENTED | 200 | PASS |
| studio | `POST /avatar-items` | MISSING | 404 | NOT TESTED |
| studio | `MOUNT /` | NOT VERIFIED | — | NOT TESTED |
| studio | `MOUNT /` | NOT VERIFIED | — | NOT TESTED |
| www | `ALL *` | MISSING | 404 | NOT TESTED |
| www | `GET /docs/openapi/:service` | MISSING | 404 | NOT TESTED |
| www | `GET /docs` | MISSING | 404 | NOT TESTED |
| www | `GET /privacy` | MISSING | 404 | NOT TESTED |
| www | `PUT /api/staff/lists/:slug` | MISSING | 404 | NOT TESTED |
| www | `GET /api/staff/lists/:slug` | MISSING | 404 | NOT TESTED |
| www | `GET /api/staff/lists` | MISSING | 404 | NOT TESTED |
| www | `DELETE /api/staff/studio-access/:id` | MISSING | 404 | NOT TESTED |
| www | `POST /api/staff/studio-access` | MISSING | 404 | NOT TESTED |
| www | `GET /api/staff/studio-access` | MISSING | 404 | NOT TESTED |
| www | `PUT /api/staff/account-flags` | MISSING | 404 | NOT TESTED |
| www | `GET /api/staff/account-flags` | MISSING | 404 | NOT TESTED |
| www | `PUT /api/staff/sandbox` | MISSING | 404 | NOT TESTED |
| www | `GET /api/staff/sandbox` | MISSING | 404 | NOT TESTED |
| www | `POST /api/staff/players/:id/clear-dorm` | MISSING | 404 | NOT TESTED |
| www | `POST /api/staff/players/:id/clear-password` | MISSING | 404 | NOT TESTED |
| www | `POST /api/staff/players/:id/username-changes` | MISSING | 404 | NOT TESTED |
| www | `POST /api/staff/players/:id/grant-plus` | MISSING | 404 | NOT TESTED |
| www | `POST /api/staff/discord-roles/supporter-gift` | MISSING | 404 | NOT TESTED |
| www | `POST /api/staff/discord-roles/sync` | MISSING | 404 | NOT TESTED |
| www | `POST /api/staff/discord-roles/:roleId/gift-tokens` | MISSING | 404 | NOT TESTED |
| www | `POST /api/staff/online/gift-tokens` | MISSING | 404 | NOT TESTED |
| www | `POST /api/staff/rooms/:roomId/gift-tokens` | MISSING | 404 | NOT TESTED |
| www | `POST /api/staff/players/:id/gift-xp` | MISSING | 404 | NOT TESTED |
| www | `POST /api/staff/players/:id/gift-item` | MISSING | 404 | NOT TESTED |
| www | `POST /api/staff/players/:id/gift-tokens` | MISSING | 404 | NOT TESTED |
| www | `GET /api/staff/players/:id/linked` | MISSING | 404 | NOT TESTED |
| www | `GET /api/staff/players/:id` | MISSING | 404 | NOT TESTED |
| www | `GET /api/staff/bans` | MISSING | 404 | NOT TESTED |
| www | `POST /api/staff/reports/:id/ban` | MISSING | 404 | NOT TESTED |
| www | `GET /api/staff/reports/:id` | MISSING | 404 | NOT TESTED |
| www | `POST /api/staff/reports` | MISSING | 404 | NOT TESTED |
| www | `GET /api/staff/reports` | MISSING | 404 | NOT TESTED |
| www | `GET /api/staff/reports/top-reported` | MISSING | 404 | NOT TESTED |
| www | `GET /api/studio-access` | MISSING | 404 | NOT TESTED |
| www | `POST /api/account/delete` | MISSING | 404 | NOT TESTED |
| www | `POST /api/benefits/claim` | MISSING | 404 | NOT TESTED |
| www | `POST /api/benefits/unlink` | MISSING | 404 | NOT TESTED |
| www | `GET /api/benefits/status` | MISSING | 404 | NOT TESTED |
| www | `POST /api/login/discord` | MISSING | 404 | NOT TESTED |
| www | `POST /api/signup` | MISSING | 404 | NOT TESTED |
| www | `GET /api/stats/online` | MISSING | 404 | NOT TESTED |
| www | `GET /server-status` | MISSING | 404 | NOT TESTED |
| www | `GET /api/config` | MISSING | 404 | NOT TESTED |
