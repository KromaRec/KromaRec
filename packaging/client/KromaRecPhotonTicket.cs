// Optional source overlay for RecFlare/patch 68f42eb6614e4fa788603bc0dd8124527c382e30.
// The 20230414 interop binding requires validation on a real installation.
using System;
using System.Reflection;
using System.Text.Json;
using BestHTTP;
using HarmonyLib;
using Il2CppInterop.Runtime;
using Photon.Realtime;

namespace RecNetPlugin.Patches;

internal static class KromaRecPhotonTicket
{
    private static readonly object Sync = new();
    private static string ticket;
    private static long expires;
    private static long generation;

    internal static void Install(Harmony harmony)
    {
        if (string.IsNullOrWhiteSpace(Plugin.KromaRecMatchHost.Value))
            return;
        var prefix = new HarmonyMethod(typeof(KromaRecPhotonTicket), nameof(Authenticate));
        var regular = AccessTools.Method(typeof(LoadBalancingPeer), "OpAuthenticate",
            new[] { typeof(string), typeof(string), typeof(AuthenticationValues), typeof(string), typeof(bool) });
        var once = AccessTools.Method(typeof(LoadBalancingPeer), "OpAuthenticateOnce",
            new[] { typeof(string), typeof(string), typeof(AuthenticationValues), typeof(string),
                    typeof(EncryptionMode), typeof(ExitGames.Client.Photon.ConnectionProtocol) });
        if (regular == null && once == null)
        {
            Plugin.Log.LogWarning("[KromaRec] Photon authentication binding unavailable");
            return;
        }
        foreach (MethodInfo method in new[] { regular, once })
            if (method != null)
                harmony.Patch(method, prefix: prefix);
        Plugin.Log.LogInfo($"[KromaRec] Photon authentication bindings: regular={regular != null}, once={once != null}");
    }

    internal static void Observe(HTTPRequest request)
    {
        var uri = request.Uri;
        if (uri.Scheme != "https" ||
            !string.Equals(uri.Host, Plugin.KromaRecMatchHost.Value, StringComparison.OrdinalIgnoreCase) ||
            (uri.AbsolutePath != "/player/connection-info" && uri.AbsolutePath != "/player/logout"))
            return;
        long attempt;
        lock (Sync)
        {
            attempt = ++generation;
            if (uri.AbsolutePath == "/player/logout")
            {
                ticket = null;
                expires = 0;
                return;
            }
        }
        var original = request.Callback;
        request.Callback = DelegateSupport.ConvertDelegate<OnRequestFinishedDelegate>(
            (Action<HTTPRequest, HTTPResponse>)((req, resp) =>
            {
                try
                {
                    if (resp != null && resp.StatusCode == 200)
                        Cache(resp.DataAsText, attempt);
                }
                catch (Exception)
                {
                    Plugin.Log.LogWarning("[KromaRec] Could not read Photon connection information");
                }
                original?.Invoke(req, resp);
            }));
    }

    private static void Cache(string json, long attempt)
    {
        using var response = JsonDocument.Parse(json);
        var root = response.RootElement;
        if (!root.GetProperty("success").GetBoolean())
            return;
        var value = root.GetProperty("value");
        var app = value.GetProperty("photonRealtimeAppId").GetString();
        var candidate = value.GetProperty("photonAuthToken").GetString();
        if (!string.Equals(app, Plugin.AppIdRT.Value, StringComparison.OrdinalIgnoreCase) ||
            candidate == null || candidate.Length > 8192)
            return;
        var parts = candidate.Split('.');
        if (parts.Length != 3)
            return;
        var payload = parts[1].Replace('-', '+').Replace('_', '/');
        payload = payload.PadRight((payload.Length + 3) / 4 * 4, '=');
        using var claims = JsonDocument.Parse(Convert.FromBase64String(payload));
        var end = claims.RootElement.GetProperty("exp").GetInt64();
        lock (Sync)
        {
            if (attempt == generation)
            {
                ticket = candidate;
                expires = end;
            }
        }
    }

    private static bool Authenticate(LoadBalancingPeer __instance, string __0,
                                     ref AuthenticationValues __2, ref bool __result)
    {
        if (!Plugin.EnableAdvancedSettings.Value || string.IsNullOrEmpty(Plugin.AppIdRT.Value) ||
            !string.Equals(__0, Plugin.AppIdRT.Value, StringComparison.OrdinalIgnoreCase) ||
            !string.Equals(__instance.ServerAddress,
                $"{Plugin.PhotonHostname.Value}:{(Plugin.PhotonPort.Value == 0 ? 4533 : Plugin.PhotonPort.Value)}",
                StringComparison.OrdinalIgnoreCase))
            return true;

        string active;
        lock (Sync)
            active = expires > DateTimeOffset.UtcNow.ToUnixTimeSeconds() ? ticket : null;
        if (active == null || !__instance.IsEncryptionAvailable)
        {
            Plugin.Log.LogWarning("[KromaRec] NameServer auth blocked: no fresh ticket or no encrypted channel");
            __result = false;
            return false;
        }
        // Change only the fresh NameServer request. Native Master and Game handoffs
        // keep the AuthenticationValues and opaque token managed by the game.
        __2 = new AuthenticationValues(__2?.UserId ?? string.Empty)
        {
            AuthType = CustomAuthenticationType.Custom,
            AuthGetParameters = "token=" + active
        };
        return true;
    }
}
