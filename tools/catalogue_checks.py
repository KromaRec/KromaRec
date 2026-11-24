"""Exercise 0.10.2 read contracts against a running smoke-test instance."""
import json
import sqlite3


def check(request, database, access_token, expired_token):
    def get(service, path, **headers):
        status, meta, body = request(service, path, **headers)
        assert status == 200 and meta["Content-Type"] == "application/json", (service, path, status)
        return json.loads(body)

    def status(service, path, expected, **headers):
        assert request(service, path, **headers)[0] == expected, (service, path, expected)

    for service, path in (("assistant", "/gameai/user/access"), ("cards", "/iam/me/channels/Announcements")):
        for token in ("", "invalid", expired_token):
            status(service, path, 401, Authorization="Bearer " + token)
    auth = {"Authorization": "Bearer " + access_token}
    assert get("assistant", "/gameai/user/access?roomId=1", **auth) == {
        "success": False, "error_id": "AI.RoomDoesNotSupportGameAI",
        "error": "This room does not support Rec Room Game AI",
    }
    for channel in ("Announcements", "JustInTimeTutorials", "None"):
        assert get("cards", "/iam/me/channels/" + channel, **auth) == []
    status("cards", "/iam/me/channels/Other", 401)
    status("cards", "/iam/me/channels/Other", 400, **auth)
    assert get("api", "/iam/me/channels/Announcements") == []
    assert get("commerce", "/purchase/v1/hasspentmoney") is False

    base = "/sections/bulk"
    first = "Rooms_New_PlayHighlight_TabsTest_Explore"
    second = "RoomCategories_MoodPlaylists_FeelingLucky"
    assert get("browse", base) == []
    selected = get("browse", f"{base}?id={second}&id=no-such-section&id={first}&id={first}")
    assert [row["id"] for row in selected] == [first, second]
    assert selected[0]["source"] == "Hot"
    path = "/curatedlists?creatorAccountId=1&type=7&name=discovery.pagesource.playexplore"
    capture = get("lists", path)
    assert capture["ListId"] == 624765592684307326 and first in capture["ItemIds"]
    assert get("lists", "/curatedlists?type=7") == capture
    assert get("lists", "/curatedlists?name=__SavedForLater_Rooms")["ListId"] == 0
    assert get("lists", "/curatedlists?type=7&name=absent")["ItemIds"] == []
    assert get("roomcomments", "/comments/get/1") == []
    status("clubs", "/club/1", 404)
    for suffix, code, msg in (("", 400, "Invalid build ID."), ("?cloudBuildId=01",400,"Invalid build ID."),
                              ("?cloudBuildId=7",404,"No such build.")):
        s, _, body = request("studio", "/cloud-builds" + suffix)
        assert s == code and json.loads(body) == {"success":False,"value":None,"error":msg}

    with sqlite3.connect(database) as db:
        db.execute("INSERT INTO club VALUES (?,?)", (1,json.dumps({"ClubId":1,"Name":"Example","CustomTags":["internal"]})))
        owned = dict(capture, ListId=9876543210, ItemIds=["local"])
        db.execute("INSERT INTO curated_list VALUES (?,?,?,?)", (1,7,capture["Name"],json.dumps(owned)))
        db.execute("INSERT INTO studio_cloud_build VALUES ('7','start','end',NULL,'asset',15)")
        db.executemany("INSERT INTO room_comment(room_id,subroom_id,player_id,message,position_x,created_at) VALUES (1,?,15,?,0.5,'start')", [(1,"one"),(2,"two"),(1,"three")])
    assert get("clubs", "/club/1") == {"ClubId":1,"Name":"Example"}
    assert get("lists", path) == owned
    assert get("studio", "/cloud-builds?cloudBuildId=7") == {"success":True,"error":None,"value":{
        "cloudBuildId":"7","startedAt":"start","completedAt":"end","error":None,"unityAssetId":"asset",
        "createdByAccountId":15,"serverDataFileName":"studio-builds/7/server-data.json"}}
    comments = get("roomcomments", "/comments/get/1?minId=1&subRoomId=1&count=1")
    assert len(comments) == 1 and comments[0]["Message"] == "three" and comments[0]["PositionX"] == 0.5
    assert comments[0]["Unread"] is True and comments[0]["AccountId"] == 15
    assert [c["CommentId"] for c in get("roomcomments", "/comments/get/1")] == [3,2,1]
    for service, path in [("lists","/curatedlists"),("browse",base),("studio","/cloud-builds"),
                          ("clubs","/club/1"),("roomcomments","/comments/get/1"),
                          ("commerce","/purchase/v1/hasspentmoney"),("assistant","/gameai/user/access"),
                          ("cards","/iam/me/channels/Announcements")]:
        assert request(service,path,"POST",**auth)[0] == 404
