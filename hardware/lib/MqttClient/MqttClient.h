#if !defined(MQTT_CLIENT_H)
#define MQTT_CLIENT_H

#include <Arduino.h>
#include <PubSubClient.h>

class MqttClient
{
private:
    PubSubClient &mqttClient;
    const char *mqttBroker;
    const unsigned int mqttPort;
    const char *mqttTopic;
    unsigned long lastReconnectAttempt;
    const unsigned int mqttKeepAlive;
    const unsigned int mqttReconnectTimeout;
    const char *mqttUsername;
    const char *mqttPassword;
    const bool isAuthenticated;

    bool reconnect();

public:
    MqttClient(
        PubSubClient &mqttClient,
        const char *brokerAddress,
        unsigned int brokerPort,
        const char *topic,
        unsigned int messageSize,
        unsigned int mqttKeepAlive,
        unsigned int mqttReconnectTimeout
        );
    MqttClient(
        PubSubClient &mqttClient,
        const char *brokerAddress,
        unsigned int brokerPort,
        const char *topic,
        unsigned int messageSize,
        unsigned int mqttKeepAlive,
        unsigned int mqttReconnectTimeout,
        const char *clientUsername,
        const char *clientPassword
        );
    ~MqttClient();

    bool connect();
    bool isConnected();
    void disconnect();
    bool publish(const char *topic, const char *payload);
    bool publish(const char *topic, const char *payload, bool retained);
    bool publishToDefaultTopic(const char *subtopic, const char *payload);
    bool publishToDefaultTopic(const char *subtopic, const char *payload, bool retained);
    void loop();
    void setCallback(std::function<void(char *, uint8_t *, unsigned int)> callback);
    bool subscribe(const char *topic);
};

#endif // MQTT_CLIENT_H
