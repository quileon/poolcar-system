import type L from 'leaflet';

const GEOAPIFY_API_KEY = "e0f80f7132454023b038a039b4d8c962"

export function panToMarker(
	map: L.Map,
	marker: L.Marker,
	latitude: number,
	longitude: number,
) {
	if (!Number.isNaN(latitude) && !Number.isNaN(longitude)) {
		const newLatLng: L.LatLngExpression = [latitude, longitude];
		marker.setLatLng(newLatLng);
		map.panTo(newLatLng);
	}
}

export function updateMarker(
	marker: L.Marker,
	latitude: number,
	longitude: number,
) {
	if (!Number.isNaN(latitude) && !Number.isNaN(longitude)) {
		const newLatLng: L.LatLngExpression = [latitude, longitude];
		marker.setLatLng(newLatLng);
	}
}

export function getTruckIcon(L: typeof import('leaflet'), colors: string[], trackerId: number) {
	const iconColor = colors[trackerId % colors.length]?.replace("#", "%23");
	return L.icon({
		iconUrl: `https://api.geoapify.com/v2/icon/?type=material&color=${iconColor}&size=42&icon=truck&iconType=awesome&contentSize=15&scaleFactor=2&apiKey=${GEOAPIFY_API_KEY}`,
		iconSize: [31, 46],
		iconAnchor: [15.5, 42],
		popupAnchor: [0, -40],
	});
}

export function getTruckIconParse(L: typeof import('leaflet'), colors: string[], trackerId: string) {
	const parsedTrackerId = parseInt(trackerId, 10);
	const iconColor = colors[parsedTrackerId % colors.length]?.replace(
		"#",
		"%23",
	);
	return L.icon({
		iconUrl: `https://api.geoapify.com/v2/icon/?type=material&color=${iconColor}&size=42&icon=truck&iconType=awesome&contentSize=15&scaleFactor=2&apiKey=${GEOAPIFY_API_KEY}`,
		iconSize: [31, 46],
		iconAnchor: [15.5, 42],
		popupAnchor: [0, -40],
	});
}

export function getCarIcon(L: typeof import('leaflet'), colors: string[], trackerId: number) {
	const iconColor = colors[trackerId % colors.length]?.replace("#", "%23");
	return L.icon({
		iconUrl: `https://api.geoapify.com/v2/icon/?type=material&color=${iconColor}&size=42&icon=car&iconType=awesome&contentSize=15&scaleFactor=2&apiKey=${GEOAPIFY_API_KEY}`,
		iconSize: [31, 46],
		iconAnchor: [15.5, 42],
		popupAnchor: [0, -40],
	});
}

export function getCarIconParse(L: typeof import('leaflet'), colors: string[], trackerId: string) {
	const parsedTrackerId = parseInt(trackerId, 10);
	const iconColor = colors[parsedTrackerId % colors.length]?.replace(
		"#",
		"%23",
	);
	return L.icon({
		iconUrl: `https://api.geoapify.com/v2/icon/?type=material&color=${iconColor}&size=42&icon=car&iconType=awesome&contentSize=15&scaleFactor=2&apiKey=${GEOAPIFY_API_KEY}`,
		iconSize: [31, 46],
		iconAnchor: [15.5, 42],
		popupAnchor: [0, -40],
	});
}
