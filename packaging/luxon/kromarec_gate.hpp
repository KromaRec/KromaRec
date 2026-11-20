#pragma once

#include <cstdlib>
#include <cerrno>
#include <cstdint>
#include <cstring>
#include <fcntl.h>
#include <optional>
#include <string>
#include <string_view>
#include <sys/socket.h>
#include <arpa/inet.h>
#include <poll.h>
#include <unistd.h>

namespace kromarec {
inline std::optional<std::string> authorize(std::string_view action, std::string_view app,
                                            std::string_view ticket, std::string_view room, int* lease = nullptr) {
    const char* endpoint = std::getenv("KROMAREC_PHOTON_GATE_ADDR");
    const char* key = std::getenv("KROMAREC_PHOTON_GATE_KEY");
    if (!endpoint || !key || std::strlen(key) != 64 ||
        app.empty() || app.size() > 256 || ticket.empty() || ticket.size() > 8192 ||
        room.size() > 1024 || app.find('\0') != app.npos ||
        ticket.find('\0') != ticket.npos || room.find('\0') != room.npos)
        return std::nullopt;

    const std::string_view bind(endpoint);
    const auto port_string = bind.substr(bind.find(':') + 1);
    if (!bind.starts_with("127.0.0.1:") || port_string.empty() ||
        port_string.size()>5 || port_string.find_first_not_of("0123456789")!=port_string.npos)
        return std::nullopt;
    const auto port = std::atoi(port_string.data());
    if (port < 1 || port > 65535) return std::nullopt;
    sockaddr_in address{};
    address.sin_family = AF_INET;
    address.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
    address.sin_port = htons(static_cast<uint16_t>(port));

    struct Socket {
        int fd;
        ~Socket() { if (fd >= 0) close(fd); }
    } socket_handle{socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC | SOCK_NONBLOCK, 0)};
    const int fd = socket_handle.fd;
    if (fd < 0) return std::nullopt;
    // Fail closed quickly when the local decision service is unavailable.
    if (connect(fd, reinterpret_cast<sockaddr*>(&address), sizeof(address)) != 0) {
        pollfd ready{fd, POLLOUT, 0};
        int error = 0;
        socklen_t size = sizeof(error);
        if (errno != EINPROGRESS || poll(&ready, 1, 100) != 1 ||
            getsockopt(fd, SOL_SOCKET, SO_ERROR, &error, &size) != 0 || error != 0)
            return std::nullopt;
    }
    if (fcntl(fd, F_SETFL, 0) != 0) return std::nullopt;
    timeval deadline{0, 300000};
    if (setsockopt(fd, SOL_SOCKET, SO_RCVTIMEO, &deadline, sizeof(deadline)) != 0 ||
        setsockopt(fd, SOL_SOCKET, SO_SNDTIMEO, &deadline, sizeof(deadline)) != 0)
        return std::nullopt;

    std::string request(key);
    for (auto field : {action, app, ticket, room}) {
        request.push_back('\0');
        request.append(field);
    }
    if (lease) {
        const char prefix[]{0, static_cast<char>(request.size() >> 8), static_cast<char>(request.size())};
        request.insert(0, prefix, sizeof(prefix));
    }
    if (send(fd, request.data(), request.size(), MSG_NOSIGNAL) != static_cast<ssize_t>(request.size()) ||
        (!lease && shutdown(fd, SHUT_WR) != 0))
        return std::nullopt;
    char reply[32];
    std::string id;
    for (;;) {
        const auto size = recv(fd, reply, sizeof(reply), 0);
        if (size < 0 || id.size() + size > 17) return std::nullopt;
        if (size == 0) {
            if (lease) return std::nullopt;
            break;
        }
        id.append(reply, static_cast<size_t>(size));
        if (lease && id.back() == '\n') {
            id.pop_back();
            break;
        }
    }
    if (id.empty() || id.front() == '0' || id.find_first_not_of("0123456789") != id.npos)
        return std::nullopt;
    if (lease) {
        if (*lease >= 0) close(*lease);
        *lease = fd;
        socket_handle.fd = -1;
    }
    return id;
}

inline std::string_view ticket_from_parameters(std::string_view parameters) {
    if (parameters.starts_with("token=")) {
        parameters.remove_prefix(6);
        return parameters.substr(0, parameters.find('&'));
    }
    // Also accept a bare compact JWT. Other parameter names fail closed.
    return parameters;
}
} // namespace kromarec
