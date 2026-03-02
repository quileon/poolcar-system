#if !defined(GPS_CLIENT_H)
#define GPS_CLIENT_H

#include <TinyGPS++.h>
#include <ArduinoJson.h>

class GpsClient
{
private:
    TinyGPSPlus &gps;
    JsonDocument jsonDoc;

public:
    GpsClient(TinyGPSPlus &gps);
    ~GpsClient();

    // Core methods
    void encode(char c);

    // Return a reference to the per-instance JSON document containing the latest GPS snapshot.
    JsonDocument &toJsonDocument();
    String toJsonString();

    // Convenience methods for common data
    bool hasValidLocation() const { return gps.location.isValid(); }
    bool hasValidTime() const { return gps.date.isValid() && gps.time.isValid(); }
    bool hasValidSpeed() const { return gps.speed.isValid(); }
    bool hasValidAltitude() const { return gps.altitude.isValid(); }

    double getLatitude() const { return gps.location.lat(); }
    double getLongitude() const { return gps.location.lng(); }
    double getSpeedKmph() const { return gps.speed.kmph(); }
    double getAltitudeMeters() const { return gps.altitude.meters(); }
    int getSatelliteCount() const { return gps.satellites.value(); }

    // Statistics
    uint32_t charsProcessed() const { return gps.charsProcessed(); }
    uint32_t sentencesWithFix() const { return gps.sentencesWithFix(); }
    uint32_t failedChecksum() const { return gps.failedChecksum(); }
    uint32_t passedChecksum() const { return gps.passedChecksum(); }
};

#endif // GPS_CLIENT_H
