#include "HttpWrapper.h"

HttpWrapper::HttpWrapper(Client& client, const char* host, uint16_t port)
    : _httpClient(client, host, port) {
}

bool HttpWrapper::post(const char* path, const char* contentType, const char* body) {
    Serial.printf("POST %s\n", path);
    _httpClient.post(path, contentType, body);

    int statusCode = _httpClient.responseStatusCode();
    String response = _httpClient.responseBody();

    Serial.print("Status code: ");
    Serial.println(statusCode);
    Serial.print("Response: ");
    Serial.println(response);
    
    // Explicitly stop the client to prevent persistent failure states (like status -3)
    // and force a fresh connection for the next request.
    _httpClient.stop();
    
    // Return true for 2xx status codes
    return (statusCode >= 200 && statusCode < 300);
}
