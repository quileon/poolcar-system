#ifndef HTTP_WRAPPER_H
#define HTTP_WRAPPER_H

#include <Arduino.h>
#include <Client.h>
#include <ArduinoHttpClient.h>

class HttpWrapper {
public:
    HttpWrapper(Client& client, const char* host, uint16_t port);
    
    // Perform a POST request
    bool post(const char* path, const char* contentType, const char* body);

private:
    HttpClient _httpClient;
};

#endif // HTTP_WRAPPER_H
