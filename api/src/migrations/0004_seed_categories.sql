-- Seed category taxonomy with benchmark CO2e averages
INSERT OR IGNORE INTO categories (slug, name, parent_id, avg_co2e_kg, avg_co2e_scope, factor_source) VALUES
  ('electronics',           'Electronics',        NULL, 30.0, 'lifecycle', 'ecoinvent-3.9'),
  ('electronics/smartphones','Smartphones',       1,    42.0, 'lifecycle', 'ecoinvent-3.9+iNEMI'),
  ('electronics/laptops',   'Laptops',            1,    28.0, 'lifecycle', 'ecoinvent-3.9'),
  ('electronics/tablets',   'Tablets',            1,    30.0, 'lifecycle', 'ecoinvent-3.9'),
  ('electronics/headphones','Headphones',         1,    22.0, 'lifecycle', 'ecoinvent-3.9'),
  ('electronics/cameras',   'Cameras',            1,    25.0, 'lifecycle', 'ecoinvent-3.9'),
  ('clothing',              'Clothing',           NULL, 12.0, 'lifecycle', 'Higg-MSI-2023'),
  ('clothing/tshirts',      'T-Shirts',           7,     9.5, 'lifecycle', 'Higg-MSI-2023'),
  ('clothing/jeans',        'Jeans',              7,    23.0, 'lifecycle', 'Higg-MSI-2023'),
  ('clothing/shoes',        'Shoes',              7,    14.0, 'lifecycle', 'Higg-MSI-2023'),
  ('clothing/jackets',      'Jackets & Coats',    7,    18.0, 'lifecycle', 'Higg-MSI-2023'),
  ('appliances',            'Appliances',         NULL,  6.0, 'lifecycle', 'ecoinvent-3.9'),
  ('appliances/washing-machine','Washing Machines',12,   6.5, 'lifecycle', 'ecoinvent-3.9'),
  ('appliances/refrigerator','Refrigerators',     12,   5.8, 'lifecycle', 'ecoinvent-3.9'),
  ('furniture',             'Furniture',          NULL,  4.0, 'lifecycle', 'ecoinvent-3.9'),
  ('furniture/chair',       'Chairs',             15,   4.2, 'lifecycle', 'ecoinvent-3.9'),
  ('furniture/desk',        'Desks',              15,   3.8, 'lifecycle', 'ecoinvent-3.9'),
  ('general',               'General',            NULL, 10.0, 'lifecycle', 'ecoinvent-3.9');

-- Seed 30 catalogue products across categories
INSERT OR IGNORE INTO products (name, brand, category_id, co2e_kg, co2e_scope, co2e_source, co2e_confidence, weight_kg, origin_country, certifications, is_active) VALUES
  -- Smartphones
  ('Fairphone 5',          'Fairphone',    2, 29.4, 'lifecycle', 'certified', 0.95, 0.213, 'NL', '["B-Corp","Fairtrade"]', 1),
  ('iPhone 15',            'Apple',        2, 59.0, 'lifecycle', 'certified', 0.95, 0.171, 'CN', '[]', 1),
  ('Galaxy S24',           'Samsung',      2, 67.0, 'lifecycle', 'certified', 0.92, 0.167, 'KR', '[]', 1),
  ('Pixel 8',              'Google',       2, 52.0, 'lifecycle', 'certified', 0.90, 0.187, 'CN', '[]', 1),
  ('Nokia G22',            'Nokia',        2, 35.0, 'lifecycle', 'estimated', 0.70, 0.195, 'CN', '["Repairability"]', 1),
  -- Laptops
  ('MacBook Air M3 13"',   'Apple',        3, 171.0,'lifecycle', 'certified', 0.95, 1.240, 'CN', '[]', 1),
  ('Framework Laptop 13',  'Framework',    3, 128.0,'lifecycle', 'certified', 0.88, 1.300, 'TW', '["Repairability"]', 1),
  ('ThinkPad X1 Carbon',   'Lenovo',       3, 156.0,'lifecycle', 'certified', 0.90, 1.120, 'CN', '[]', 1),
  -- Headphones
  ('Sony WH-1000XM5',      'Sony',         5, 18.4, 'lifecycle', 'certified', 0.88, 0.250, 'CN', '[]', 1),
  ('AirPods Pro 2',        'Apple',        5, 28.0, 'lifecycle', 'certified', 0.92, 0.062, 'CN', '[]', 1),
  -- T-Shirts
  ('Organic Tee',          'Patagonia',    8,  5.8, 'lifecycle', 'certified', 0.90, 0.230, 'US', '["GOTS","B-Corp"]', 1),
  ('Classic Tee',          'H&M',          8, 11.2, 'lifecycle', 'estimated', 0.65, 0.200, 'BD', '[]', 1),
  ('Heritage Tee',         'Pact',         8,  4.9, 'lifecycle', 'certified', 0.88, 0.210, 'IN', '["GOTS","Fair Trade"]', 1),
  -- Jeans
  ('501 Original',         'Levi''s',      9, 33.4, 'lifecycle', 'certified', 0.88, 0.680, 'BD', '[]', 1),
  ('Organic Slim Jeans',   'Nudie Jeans',  9, 21.0, 'lifecycle', 'certified', 0.90, 0.650, 'DE', '["GOTS"]', 1),
  -- Shoes
  ('Stan Smith Mylo',      'Adidas',      10, 10.5, 'lifecycle', 'certified', 0.85, 0.350, 'DE', '["Vegan"]', 1),
  ('Classic Vans Old Skool','Vans',        10, 19.2, 'lifecycle', 'estimated', 0.65, 0.400, 'CN', '[]', 1),
  -- Jackets
  ('Better Sweater Fleece','Patagonia',   11, 11.2, 'lifecycle', 'certified', 0.88, 0.470, 'US', '["recycled","B-Corp"]', 1),
  ('Nano Puff Jacket',     'Patagonia',   11, 18.9, 'lifecycle', 'certified', 0.88, 0.340, 'VN', '["recycled","B-Corp"]', 1),
  ('Down Jacket',          'Uniqlo',      11, 22.0, 'lifecycle', 'estimated', 0.65, 0.500, 'CN', '[]', 1),
  -- Washing machines
  ('EcoLine 9000',         'Miele',       13, 312.0,'lifecycle', 'manual', 0.80, 68.0, 'DE', '["Energy A"]', 1),
  ('Series 6 WGG14200',    'Bosch',       13, 328.0,'lifecycle', 'manual', 0.78, 66.0, 'DE', '["Energy A"]', 1),
  -- Refrigerators
  ('GSS30IYNFS',           'GE',          14, 320.0,'lifecycle', 'estimated', 0.70, 78.0, 'US', '["Energy Star"]', 1),
  ('KFF96APEA',            'Siemens',     14, 280.0,'lifecycle', 'manual', 0.80, 72.0, 'DE', '["Energy A++"]', 1),
  -- Chairs
  ('Aeron',                'Herman Miller',16,85.0, 'lifecycle', 'certified', 0.88, 15.5, 'US', '["GREENGUARD"]', 1),
  ('FLINTAN',              'IKEA',        16, 52.0, 'lifecycle', 'certified', 0.85, 11.0, 'PL', '["IKEA-Climate"]', 1),
  ('Mirra 2',              'Herman Miller',16,78.0, 'lifecycle', 'certified', 0.88, 14.5, 'US', '["GREENGUARD"]', 1),
  -- Desks
  ('Bekant Desk',          'IKEA',        17, 41.0, 'lifecycle', 'certified', 0.82, 48.0, 'PL', '["FSC","IKEA-Climate"]', 1),
  ('Stand Desk Pro',       'Uplift',      17, 88.0, 'lifecycle', 'estimated', 0.68, 52.0, 'TW', '["BIFMA"]', 1),
  ('Jarvis Bamboo Desk',   'Fully',       17, 62.0, 'lifecycle', 'estimated', 0.72, 45.0, 'CN', '["FSC"]', 1);
