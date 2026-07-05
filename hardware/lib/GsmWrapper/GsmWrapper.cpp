#include <../../include/main.h>
#include "GsmWrapper.h"

GsmWrapper::GsmWrapper(
    TinyGsm &gsmModem,
    const char *apn,
    const char *apnUser,
    const char *apnPassword,
    const unsigned short maxRetries,
    const unsigned long connectionCheckInterval)
    : gsmModem(gsmModem), apn(apn), apnUser(apnUser), apnPassword(apnPassword), maxRetries(maxRetries), connectionCheckInterval(connectionCheckInterval), lastConnectionCheck(0), connected(false)
{
#if defined(TINY_GSM_MODEM_SIM808)
    this->gps = false;
#endif // TINY_GSM_MODEM_SIM808
}

GsmWrapper::~GsmWrapper() = default;

bool GsmWrapper::initializeModem()
{
    if (!gsmModem.init())
    {
        Serial.println("Could not initialize GSM modem!");
        return false;
    }
    Serial.println("GSM modem initialized!");
    String modemInfo = gsmModem.getModemInfo();
    Serial.println("Using modem: " + modemInfo);
    return true;
}

bool GsmWrapper::connectGprs()
{
    if (this->gsmModem.isGprsConnected())
    {
        Serial.println("GPRS already connected!");
        return true;
    }

    Serial.println("Connecting to GPRS...");
    if (this->gsmModem.gprsConnect(apn, apnUser, apnPassword))
    {
        Serial.println("GPRS connected!");
        return true;
    }

    Serial.println("Failed to connect GPRS!");
    return false;
}

bool GsmWrapper::isActive()
{
    unsigned short retries = 0;
    while (retries < 10)
    {
        delay(100);
        if (gsmModem.testAT(1000))
        {
            return true;
        }
        retries++;
    }
    return false;
}

bool GsmWrapper::begin()
{
    unsigned short retries = 0;

    while (retries < this->maxRetries)
    {
        if (!this->initializeModem())
        {
            Serial.println("Retry initializing modem...");
            retries++;
            delay(500);
            continue;
        }

        if (!connectGprs())
        {
            Serial.println("Retry connecting to GPRS...");
            // Check later on!
            this->gsmModem.gprsDisconnect();
            retries++;
            delay(500);
            continue;
        }

        String ipAddress = this->getLocalIP();
        Serial.println("Using IP address: " + ipAddress);

        connected = true;
        lastConnectionCheck = millis();
        return true;
    }

    Serial.println("Failed to connect to GPRS!");
    connected = false;
    return false;
}

bool GsmWrapper::isConnected()
{
    return connected && this->gsmModem.isGprsConnected();
}

bool GsmWrapper::ensureConnection()
{
    // Don't reconnect when not connected if below `connectionCheckInterval`
    if (millis() - this->lastConnectionCheck < this->connectionCheckInterval)
    {
        return this->isConnected();
    }
    lastConnectionCheck = millis();

    // Check if already connected
    if (this->isConnected())
    {
        return true;
    }
    Serial.println("Reconnecting to GPRS...");
    
    // Reconnect
    if (connectGprs())
    {
        connected = true;
        return true;
    }

    connected = false;
    return false;
}

int GsmWrapper::getSignalStrength()
{
    return this->gsmModem.getSignalQuality();
}

bool GsmWrapper::getNetworkInfo(String &lac, String &ci)
{
    this->gsmModem.sendAT(GF("+CREG=2"));
    this->gsmModem.waitResponse();

    this->gsmModem.sendAT(GF("+CREG?"));
    String response;
    if (this->gsmModem.waitResponse(10000L, response) == 1) {
        int idx = response.indexOf("+CREG:");
        if (idx != -1) {
            int firstComma = response.indexOf(',', idx);
            if (firstComma != -1) {
                int secondComma = response.indexOf(',', firstComma + 1);
                if (secondComma != -1) {
                    int thirdComma = response.indexOf(',', secondComma + 1);
                    if (thirdComma != -1) {
                        lac = response.substring(secondComma + 1, thirdComma);
                        lac.replace("\"", "");
                        lac.trim();

                        int fourthComma = response.indexOf(',', thirdComma + 1);
                        int endIdx;
                        if (fourthComma != -1) {
                            endIdx = fourthComma;
                        } else {
                            endIdx = response.indexOf('\r', thirdComma + 1);
                            if (endIdx == -1) endIdx = response.indexOf('\n', thirdComma + 1);
                            if (endIdx == -1) endIdx = response.length();
                        }

                        ci = response.substring(thirdComma + 1, endIdx);
                        ci.replace("\"", "");
                        ci.trim();

                        if (lac.length() > 0 && ci.length() > 0) {
                            return true;
                        }
                    }
                }
            }
        }
    }
    return false;
}

String GsmWrapper::getLocalIP()
{
    return this->gsmModem.getLocalIP();
}

void GsmWrapper::disconnect()
{
    if (connected)
    {
        this->gsmModem.gprsDisconnect();
        connected = false;
    }
}

#if defined(TINY_GSM_MODEM_SIM808)
bool GsmWrapper::enableGPS()
{
    const bool success = this->gsmModem.enableGPS();
    if (success)
    {
        this->gps = true;
    }
    return success;
}

bool GsmWrapper::disableGPS()
{
    const bool success = this->gsmModem.disableGPS();
    if (success)
    {
        this->gps = false;
    }
    return success;
}
bool GsmWrapper::statusGPS()
{
    return this->gps;
}
String GsmWrapper::getRawGPS()
{
    return this->gsmModem.getGPSraw();
}
#endif // TINY_GSM_MODEM_SIM808