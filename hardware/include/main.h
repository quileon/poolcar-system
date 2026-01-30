#if !defined(MAIN_H)
#define MAIN_H

#include <Arduino.h>

// TinyGSM config
// #define TINY_GSM_MODEM_SIM900
#define TINY_GSM_MODEM_SIM808
#define TINY_GSM_RX_BUFFER 1024
// #define TINY_GSM_AUTOBAUD
#define TINY_GSM_AUTOBAUD_MINIMUM 9600
#define TINY_GSM_AUTOBAUD_MAXIMUM 115200

// Stream Debugger config
// #define DUMP_AT_COMMANDS

// GSM Configuration
// -- 3 APN Settings
constexpr char APN[] = "tri.co.id";
constexpr char APN_USER[] = "3gprs";
constexpr char APN_PASSWORD[] = "3gpr";

// -- by.U APN Settings
// constexpr char APN[] = "internet";
// constexpr char APN_USER[] = "";
// constexpr char APN_PASSWORD[] = "";

constexpr short GSM_MAX_RETRIES = 3;

// MQTT Configuration
constexpr char MQTT_BROKER[] = "w309506f.ala.eu-central-1.emqxsl.com";
constexpr int MQTT_PORT = 8883;
// constexpr char MQTT_BROKER[] = "broker.emqx.io";
// constexpr int MQTT_PORT = 1883;
constexpr char MQTT_TOPIC[] = "poolcar/";
constexpr int MQTT_MESSAGE_SIZE = 1024;
constexpr int MQTT_KEEP_ALIVE_TIMEOUT = 30000;
constexpr int MQTT_RECONNECT_TIMEOUT = 5000;
constexpr char MQTT_USERNAME[] = "tracker";
constexpr char MQTT_PASSWORD[] = "rahasia";
constexpr char MQTT_CA_CERTIFICATE[] PROGMEM = R"(
-----BEGIN CERTIFICATE-----
MIIDjjCCAnagAwIBAgIQAzrx5qcRqaC7KGSxHQn65TANBgkqhkiG9w0BAQsFADBh
MQswCQYDVQQGEwJVUzEVMBMGA1UEChMMRGlnaUNlcnQgSW5jMRkwFwYDVQQLExB3
d3cuZGlnaWNlcnQuY29tMSAwHgYDVQQDExdEaWdpQ2VydCBHbG9iYWwgUm9vdCBH
MjAeFw0xMzA4MDExMjAwMDBaFw0zODAxMTUxMjAwMDBaMGExCzAJBgNVBAYTAlVT
MRUwEwYDVQQKEwxEaWdpQ2VydCBJbmMxGTAXBgNVBAsTEHd3dy5kaWdpY2VydC5j
b20xIDAeBgNVBAMTF0RpZ2lDZXJ0IEdsb2JhbCBSb290IEcyMIIBIjANBgkqhkiG
9w0BAQEFAAOCAQ8AMIIBCgKCAQEAuzfNNNx7a8myaJCtSnX/RrohCgiN9RlUyfuI
2/Ou8jqJkTx65qsGGmvPrC3oXgkkRLpimn7Wo6h+4FR1IAWsULecYxpsMNzaHxmx
1x7e/dfgy5SDN67sH0NO3Xss0r0upS/kqbitOtSZpLYl6ZtrAGCSYP9PIUkY92eQ
q2EGnI/yuum06ZIya7XzV+hdG82MHauVBJVJ8zUtluNJbd134/tJS7SsVQepj5Wz
tCO7TG1F8PapspUwtP1MVYwnSlcUfIKdzXOS0xZKBgyMUNGPHgm+F6HmIcr9g+UQ
vIOlCsRnKPZzFBQ9RnbDhxSJITRNrw9FDKZJobq7nMWxM4MphQIDAQABo0IwQDAP
BgNVHRMBAf8EBTADAQH/MA4GA1UdDwEB/wQEAwIBhjAdBgNVHQ4EFgQUTiJUIBiV
5uNu5g/6+rkS7QYXjzkwDQYJKoZIhvcNAQELBQADggEBAGBnKJRvDkhj6zHd6mcY
1Yl9PMWLSn/pvtsrF9+wX3N3KjITOYFnQoQj8kVnNeyIv/iPsGEMNKSuIEyExtv4
NeF22d+mQrvHRAiGfzZ0JFrabA0UWTW98kndth/Jsw1HKj2ZL7tcu7XUIOGZX1NG
Fdtom/DzMNU+MeKNhJ7jitralj41E6Vf8PlwUHBHQRFXGU7Aj64GxJUTFy8bJZ91
8rGOmaFvE7FBcf6IKshPECBV1/MUReXgRPTqh5Uykw7+U0b6LJ3/iyK5S9kJRaTe
pLiaWN0bfVKfjllDiIGknibVb63dDcY3fe0Dkhvld1927jyNxF1WW6LZZm6zNTfl
MrY=
-----END CERTIFICATE-----
)";
#define MQTT_SECURE

// Pin Configuration
constexpr int GSM_RX_PIN = 16; // Connect to TX of GSM module
constexpr int GSM_TX_PIN = 17; // Connect to RX of GSM module
constexpr int GPS_RX_PIN = 22; // Connect to TX of GPS module
constexpr int GPS_TX_PIN = 23; // Connect to RX of GPS module
constexpr int ESP32_COMMUNICATION_BAUD_RATE = 9600;

// App Configuration
constexpr timer_t PUBLISH_INTERVAL = 1000;
constexpr int TRACKER_ID = 1;

#endif // MAIN_H
