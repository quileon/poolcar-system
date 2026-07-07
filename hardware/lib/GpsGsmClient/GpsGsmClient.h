#include <../../include/main.h>

#if !defined(GPS_GSM_CLIENT_H)
#define GPS_GSM_CLIENT_H

#if defined(TINY_GSM_MODEM_SIM808)

#include <ArduinoJson.h>
#include <TinyGSM.h>

#define _GPS_EARTH_MEAN_RADIUS 6371009 // old: 6372795

class GpsGsmClient
{
private:
    static constexpr double _GPS_MPH_PER_KNOT = 1.15077945;
    static constexpr double _GPS_MPS_PER_KNOT = 0.51444444;
    static constexpr double _GPS_KMPH_PER_KNOT = 1.852;
    static constexpr double _GPS_MILES_PER_METER = 0.00062137112;
    static constexpr double _GPS_KM_PER_METER = 0.001;
    static constexpr double _GPS_FEET_PER_METER = 3.2808399;
    static constexpr int _GPS_MAX_FIELD_SIZE = 15;

    TinyGsm &gsmModem;
    JsonDocument jsonDoc;

    // Stores the last raw GPS string to detect stale fixes
    String _lastRawGPS;

public:
    GpsGsmClient(TinyGsm &gsmModem);
    ~GpsGsmClient();

    // Core methods
    JsonDocument &toJsonDocument();
    String toJsonString();

    // Convenience methods
    float meters(const float &meters);
    float miles(const float &meters);
    float kilometers(const float &meters);
    float feet(const float &meters);

    float knots(const float &knots);
    float mph(const float &knots);
    float mps(const float &knots);
    float kmph(const float &knots);
};
#endif // TINY_GSM_MODEM_SIM808
#endif // GPS_GSM_CLIENT_H
