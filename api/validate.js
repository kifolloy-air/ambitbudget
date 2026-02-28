export default async function handler(req, res) {
  // CORS headers — allow requests from local file:// and any origin
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'POST, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type');
  if (req.method === 'OPTIONS') return res.status(200).end();
  if (req.method !== 'POST') return res.status(405).json({ error: 'Method not allowed' });

  const { license_key, instance_name } = req.body || {};
  if (!license_key) return res.status(400).json({ error: 'Missing license_key' });

  try {
    const response = await fetch('https://api.lemonsqueezy.com/v1/licenses/activate', {
      method: 'POST',
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/x-www-form-urlencoded'
      },
      body: new URLSearchParams({
        license_key,
        instance_name: instance_name || 'Ambit Budget Pro'
      })
    });
    const data = await response.json();
    res.status(200).json(data);
  } catch (e) {
    res.status(500).json({ error: 'License validation failed. Please try again.' });
  }
}
