#if !defined(GSM_WRAPPER_H)
#define GSM_WRAPPER_H

#include <Arduino.h>
#include <TinyGsmClient.h>

class GsmWrapper
{
private:
    TinyGsm &gsmModem;
    const char *apn;
    const char *apnUser;
    const char *apnPassword;
    const unsigned short maxRetries;
    const unsigned int connectionCheckInterval;

    unsigned long lastConnectionCheck;
    // Check if `void setup` fails!
    bool connected;

#if defined(TINY_GSM_MODEM_SIM808)
    bool gps = false;
#endif // TINY_GSM_MODEM_SIM808

    bool initializeModem();
    bool connectGprs();

public:
    GsmWrapper(
        TinyGsm &gsmModem,
        const char *apn,
        const char *apnUser,
        const char *apnPassword,
        unsigned short maxRetries = 5,
        unsigned long connectionCheckInterval = 5000);
    ~GsmWrapper();

    bool begin();
    bool isActive();
    bool isConnected();
    bool ensureConnection();
    int getSignalStrength();
    bool getNetworkInfo(String &lac, String &ci);
    String getLocalIP();
    void disconnect();

#if defined(TINY_GSM_MODEM_SIM808)
    bool enableGPS();
    bool disableGPS();
    bool statusGPS();
    String getRawGPS();
#endif // TINY_GSM_MODEM_SIM808
};

#endif // GSM_WRAPPER_H