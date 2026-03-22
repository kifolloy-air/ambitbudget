export default function handler(req, res) {
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Cache-Control', 'no-cache');
  res.json({
    version: '1.0.0',
    notes: 'Initial release',
    url: 'https://ambitbudget.com/app/'
  });
}
