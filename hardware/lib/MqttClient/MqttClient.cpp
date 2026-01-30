#include "MqttClient.h"

MqttClient::MqttClient(
    PubSubClient &mqttClient,
    const char *brokerAddress,
    const unsigned int brokerPort,
    const char *topic,
    const unsigned int messageSize,
    const unsigned int mqttKeepAlive,
    const unsigned int mqttReconnectTimeout
    )
    : mqttClient(mqttClient), mqttBroker(brokerAddress), mqttPort(brokerPort), mqttTopic(topic),
      lastReconnectAttempt(0), mqttKeepAlive(mqttKeepAlive), mqttReconnectTimeout(mqttReconnectTimeout), mqttUsername(nullptr), mqttPassword(nullptr), isAuthenticated(false)
{
    mqttClient.setServer(mqttBroker, mqttPort);
    mqttClient.setBufferSize(messageSize);
}

MqttClient::MqttClient(
    PubSubClient &mqttClient,
    const char *brokerAddress,
    const unsigned int brokerPort,
    const char *topic,
    const unsigned int messageSize,
    const unsigned int mqttKeepAlive,
    const unsigned int mqttReconnectTimeout,
    const char *clientUsername = nullptr,
    const char *clientPassword = nullptr
    )
    : mqttClient(mqttClient), mqttBroker(brokerAddress), mqttPort(brokerPort), mqttTopic(topic),
      lastReconnectAttempt(0), mqttKeepAlive(mqttKeepAlive), mqttReconnectTimeout(mqttReconnectTimeout), mqttUsername(clientUsername), mqttPassword(clientPassword), isAuthenticated(clientUsername != nullptr && clientPassword != nullptr)
{
    mqttClient.setServer(mqttBroker, mqttPort);
    mqttClient.setBufferSize(messageSize);
    mqttClient.setKeepAlive(mqttKeepAlive);
}

MqttClient::~MqttClient()
= default;

bool MqttClient::connect()
{
    String clientId = "tracker-";
    clientId += String(random(0xffff), HEX);

    Serial.print("Attempting MQTT connection to ");
    Serial.print(this->mqttBroker);
    Serial.print(":");
    Serial.println(this->mqttPort);

    bool connected = false;
    if (this->mqttUsername != nullptr && this->mqttPassword != nullptr)
    {
        connected = this->mqttClient.connect(clientId.c_str(), this->mqttUsername, this->mqttPassword);
    }
    else
    {
        connected = this->mqttClient.connect(clientId.c_str());
    }

    if (connected)
    {
        Serial.println("MQTT connected!");
        return true;
    }
    Serial.print("MQTT connection failed, rc=");
    Serial.print(this->mqttClient.state());
    Serial.println("!");
    return false;
}

bool MqttClient::isConnected()
{
    return this->mqttClient.connected();
}

bool MqttClient::reconnect()
{
    if (millis() - lastReconnectAttempt < this->mqttReconnectTimeout)
    {
        return false;
    }

    lastReconnectAttempt = millis();
    Serial.println("Attempting MQTT reconnection!");
    if (connect())
    {
        lastReconnectAttempt = 0;
        return true;
    }
    return false;
}

void MqttClient::disconnect()
{
    this->mqttClient.disconnect();
}

bool MqttClient::publish(const char *topic, const char *payload)
{
    return this->publish(topic, payload, false);
}

bool MqttClient::publish(const char *topic, const char *payload, bool retained)
{
    if (!this->isConnected())
    {
        if (!this->reconnect())
        {
            return false;
        }
    }

    Serial.print("Publishing to topic: ");
    Serial.println(topic);
    Serial.print("Payload: ");
    Serial.println(payload);

    bool result = this->mqttClient.publish(topic, payload, retained);
    if (!result)
    {
        Serial.println("Failed to publish message!");
        return result;
    }
    Serial.println("Message published successfully!");
    return result;
}

bool MqttClient::publishToDefaultTopic(const char *subtopic, const char *payload)
{
    String fullTopic = String(this->mqttTopic) + String(subtopic);
    return this->publish(fullTopic.c_str(), payload, false);
}

bool MqttClient::publishToDefaultTopic(const char *subtopic, const char *payload, bool retained)
{
    String fullTopic = String(this->mqttTopic) + String(subtopic);
    return this->publish(fullTopic.c_str(), payload, retained);
}

void MqttClient::loop()
{
    if (!this->isConnected())
    {
        reconnect();
    }
    else
    {
        this->mqttClient.loop();
    }
}

void MqttClient::setCallback(std::function<void(char *, uint8_t *, unsigned int)> callback)
{
    this->mqttClient.setCallback(callback);
}

bool MqttClient::subscribe(const char *topic)
{
    if (!this->isConnected())
    {
        if (!this->reconnect())
        {
            return false;
        }
    }

    Serial.print("Subscribing to topic: ");
    Serial.println(topic);

    return this->mqttClient.subscribe(topic);
}