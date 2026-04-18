-- Categories
INSERT INTO categories (slug, name, parent_id, avg_co2e_kg, avg_co2e_scope, factor_source) VALUES
  ('electronics',  'Electronics', NULL, NULL, 'lifecycle', 'USEEIO-v2.1'),
  ('clothing',     'Clothing',    NULL, NULL, 'lifecycle', 'USEEIO-v2.1'),
  ('home',         'Home & Kitchen', NULL, NULL, 'lifecycle', 'USEEIO-v2.1');

INSERT INTO categories (slug, name, parent_id, avg_co2e_kg, avg_co2e_scope, factor_source)
SELECT 'smartphones', 'Smartphones', id, 8.5,  'lifecycle', 'USEEIO-v2.1' FROM categories WHERE slug = 'electronics';
INSERT INTO categories (slug, name, parent_id, avg_co2e_kg, avg_co2e_scope, factor_source)
SELECT 'laptops',     'Laptops',     id, 44.0, 'lifecycle', 'USEEIO-v2.1' FROM categories WHERE slug = 'electronics';
INSERT INTO categories (slug, name, parent_id, avg_co2e_kg, avg_co2e_scope, factor_source)
SELECT 'headphones',  'Headphones',  id, 5.2,  'lifecycle', 'USEEIO-v2.1' FROM categories WHERE slug = 'electronics';
INSERT INTO categories (slug, name, parent_id, avg_co2e_kg, avg_co2e_scope, factor_source)
SELECT 'tshirts',     'T-Shirts',    id, 2.0,  'lifecycle', 'USEEIO-v2.1' FROM categories WHERE slug = 'clothing';
INSERT INTO categories (slug, name, parent_id, avg_co2e_kg, avg_co2e_scope, factor_source)
SELECT 'jeans',       'Jeans',       id, 13.5, 'lifecycle', 'USEEIO-v2.1' FROM categories WHERE slug = 'clothing';
INSERT INTO categories (slug, name, parent_id, avg_co2e_kg, avg_co2e_scope, factor_source)
SELECT 'shoes',       'Shoes',       id, 7.5,  'lifecycle', 'USEEIO-v2.1' FROM categories WHERE slug = 'clothing';
INSERT INTO categories (slug, name, parent_id, avg_co2e_kg, avg_co2e_scope, factor_source)
SELECT 'appliances',  'Appliances',  id, 20.0, 'lifecycle', 'USEEIO-v2.1' FROM categories WHERE slug = 'home';
INSERT INTO categories (slug, name, parent_id, avg_co2e_kg, avg_co2e_scope, factor_source)
SELECT 'furniture',   'Furniture',   id, 35.0, 'lifecycle', 'USEEIO-v2.1' FROM categories WHERE slug = 'home';

-- ── Smartphones (Tier-2: weight_kg × 42.0 kg CO₂e/kg) ───────────────────────
INSERT INTO products (name, brand, category_id, asin, url, image_url, co2e_kg, co2e_scope, co2e_source, co2e_confidence, weight_kg, origin_country, is_active)
SELECT 'Redmi 13C 5G (6GB+128GB, Startrail Black)', 'Xiaomi',
  id, 'B0CJXYS7MF', 'https://www.amazon.in/dp/B0CJXYS7MF', 'https://m.media-amazon.com/images/P/B0CJXYS7MF.01._SL500_.jpg',
  ROUND(0.190 * 42.0, 2), 'lifecycle', 'estimated', 0.65, 0.190, 'IN', 1
FROM categories WHERE slug = 'smartphones';

INSERT INTO products (name, brand, category_id, asin, url, image_url, co2e_kg, co2e_scope, co2e_source, co2e_confidence, weight_kg, origin_country, is_active)
SELECT 'Samsung Galaxy M34 5G (8GB+128GB, Midnight Blue)', 'Samsung',
  id, 'B0C4HFH1RR', 'https://www.amazon.in/dp/B0C4HFH1RR', 'https://m.media-amazon.com/images/P/B0C4HFH1RR.01._SL500_.jpg',
  ROUND(0.208 * 42.0, 2), 'lifecycle', 'estimated', 0.65, 0.208, 'IN', 1
FROM categories WHERE slug = 'smartphones';

INSERT INTO products (name, brand, category_id, asin, url, image_url, co2e_kg, co2e_scope, co2e_source, co2e_confidence, weight_kg, origin_country, is_active)
SELECT 'OnePlus Nord CE3 Lite 5G (8GB+256GB, Chromatic Gray)', 'OnePlus',
  id, 'B0C3WDGB85', 'https://www.amazon.in/dp/B0C3WDGB85', 'https://m.media-amazon.com/images/P/B0C3WDGB85.01._SL500_.jpg',
  ROUND(0.195 * 42.0, 2), 'lifecycle', 'estimated', 0.65, 0.195, 'CN', 1
FROM categories WHERE slug = 'smartphones';

INSERT INTO products (name, brand, category_id, asin, url, image_url, co2e_kg, co2e_scope, co2e_source, co2e_confidence, weight_kg, origin_country, is_active)
SELECT 'Motorola Moto G84 5G (12GB+256GB, Marshmallow Blue)', 'Motorola',
  id, 'B0CD7SM4Q9', 'https://www.amazon.in/dp/B0CD7SM4Q9', 'https://m.media-amazon.com/images/P/B0CD7SM4Q9.01._SL500_.jpg',
  ROUND(0.167 * 42.0, 2), 'lifecycle', 'estimated', 0.65, 0.167, 'CN', 1
FROM categories WHERE slug = 'smartphones';

-- ── Laptops (Tier-2: weight_kg × 28.0 kg CO₂e/kg) ───────────────────────────
INSERT INTO products (name, brand, category_id, asin, url, image_url, co2e_kg, co2e_scope, co2e_source, co2e_confidence, weight_kg, origin_country, is_active)
SELECT 'Lenovo IdeaPad Slim 3 (Intel Core i5-13420H, 16GB, 512GB SSD)', 'Lenovo',
  id, 'B0CQ5X4VJP', 'https://www.amazon.in/dp/B0CQ5X4VJP', 'https://m.media-amazon.com/images/P/B0CQ5X4VJP.01._SL500_.jpg',
  ROUND(1.50 * 28.0, 2), 'lifecycle', 'estimated', 0.65, 1.50, 'CN', 1
FROM categories WHERE slug = 'laptops';

INSERT INTO products (name, brand, category_id, asin, url, image_url, co2e_kg, co2e_scope, co2e_source, co2e_confidence, weight_kg, origin_country, is_active)
SELECT 'HP Laptop 15s-eq3325AU (AMD Ryzen 5 7520U, 8GB, 512GB SSD)', 'HP',
  id, 'B09NH7PCBQ', 'https://www.amazon.in/dp/B09NH7PCBQ', 'https://m.media-amazon.com/images/P/B09NH7PCBQ.01._SL500_.jpg',
  ROUND(1.69 * 28.0, 2), 'lifecycle', 'estimated', 0.65, 1.69, 'CN', 1
FROM categories WHERE slug = 'laptops';

INSERT INTO products (name, brand, category_id, asin, url, image_url, co2e_kg, co2e_scope, co2e_source, co2e_confidence, weight_kg, origin_country, is_active)
SELECT 'Acer Aspire Lite AL15-52 (Intel Core i5-1235U, 16GB, 512GB SSD)', 'Acer',
  id, 'B0BHFDQ1K5', 'https://www.amazon.in/dp/B0BHFDQ1K5', 'https://m.media-amazon.com/images/P/B0BHFDQ1K5.01._SL500_.jpg',
  ROUND(1.60 * 28.0, 2), 'lifecycle', 'estimated', 0.65, 1.60, 'CN', 1
FROM categories WHERE slug = 'laptops';

-- ── Headphones (Tier-2: weight_kg × 22.0 kg CO₂e/kg) ────────────────────────
INSERT INTO products (name, brand, category_id, asin, url, image_url, co2e_kg, co2e_scope, co2e_source, co2e_confidence, weight_kg, origin_country, is_active)
SELECT 'boAt Rockerz 450 Bluetooth On-Ear Headphones', 'boAt',
  id, 'B07BN12BMY', 'https://www.amazon.in/dp/B07BN12BMY', 'https://m.media-amazon.com/images/P/B07BN12BMY.01._SL500_.jpg',
  ROUND(0.250 * 22.0, 2), 'lifecycle', 'estimated', 0.65, 0.250, 'CN', 1
FROM categories WHERE slug = 'headphones';

INSERT INTO products (name, brand, category_id, asin, url, image_url, co2e_kg, co2e_scope, co2e_source, co2e_confidence, weight_kg, origin_country, is_active)
SELECT 'Sony WH-1000XM5 Wireless Noise Cancelling Headphones', 'Sony',
  id, 'B09XS7JWHH', 'https://www.amazon.in/dp/B09XS7JWHH', 'https://m.media-amazon.com/images/P/B09XS7JWHH.01._SL500_.jpg',
  ROUND(0.250 * 22.0, 2), 'lifecycle', 'estimated', 0.65, 0.250, 'CN', 1
FROM categories WHERE slug = 'headphones';

INSERT INTO products (name, brand, category_id, asin, url, image_url, co2e_kg, co2e_scope, co2e_source, co2e_confidence, weight_kg, origin_country, is_active)
SELECT 'JBL Tune 770NC Wireless Over-Ear ANC Headphones', 'JBL',
  id, 'B0B9PZ3P6J', 'https://www.amazon.in/dp/B0B9PZ3P6J', 'https://m.media-amazon.com/images/P/B0B9PZ3P6J.01._SL500_.jpg',
  ROUND(0.223 * 22.0, 2), 'lifecycle', 'estimated', 0.65, 0.223, 'CN', 1
FROM categories WHERE slug = 'headphones';

-- ── T-Shirts (Tier-2: weight_kg × 9.5 kg CO₂e/kg) ───────────────────────────
INSERT INTO products (name, brand, category_id, asin, url, image_url, co2e_kg, co2e_scope, co2e_source, co2e_confidence, weight_kg, origin_country, is_active)
SELECT "Jockey Men's Regular Fit Cotton T-Shirt", 'Jockey',
  id, 'B07XQ8QFXK', 'https://www.amazon.in/dp/B07XQ8QFXK', 'https://m.media-amazon.com/images/P/B07XQ8QFXK.01._SL500_.jpg',
  ROUND(0.200 * 9.5, 2), 'lifecycle', 'estimated', 0.60, 0.200, 'IN', 1
FROM categories WHERE slug = 'tshirts';

INSERT INTO products (name, brand, category_id, asin, url, image_url, co2e_kg, co2e_scope, co2e_source, co2e_confidence, weight_kg, origin_country, is_active)
SELECT "US Polo Assn. Men's Slim Fit T-Shirt", 'U.S. Polo Assn.',
  id, 'B08CXMCBRQ', 'https://www.amazon.in/dp/B08CXMCBRQ', 'https://m.media-amazon.com/images/P/B08CXMCBRQ.01._SL500_.jpg',
  ROUND(0.210 * 9.5, 2), 'lifecycle', 'estimated', 0.60, 0.210, 'IN', 1
FROM categories WHERE slug = 'tshirts';

-- ── Jeans (Tier-2: weight_kg × 23.0 kg CO₂e/kg) ─────────────────────────────
INSERT INTO products (name, brand, category_id, asin, url, image_url, co2e_kg, co2e_scope, co2e_source, co2e_confidence, weight_kg, origin_country, is_active)
SELECT "Levi's Men's 511 Slim Jeans", "Levi's",
  id, 'B07HNPBWNK', 'https://www.amazon.in/dp/B07HNPBWNK', 'https://m.media-amazon.com/images/P/B07HNPBWNK.01._SL500_.jpg',
  ROUND(0.620 * 23.0, 2), 'lifecycle', 'estimated', 0.60, 0.620, 'IN', 1
FROM categories WHERE slug = 'jeans';

INSERT INTO products (name, brand, category_id, asin, url, image_url, co2e_kg, co2e_scope, co2e_source, co2e_confidence, weight_kg, origin_country, is_active)
SELECT "Wrangler Men's Regular Fit Jeans", 'Wrangler',
  id, 'B07PB7J8HV', 'https://www.amazon.in/dp/B07PB7J8HV', 'https://m.media-amazon.com/images/P/B07PB7J8HV.01._SL500_.jpg',
  ROUND(0.580 * 23.0, 2), 'lifecycle', 'estimated', 0.60, 0.580, 'IN', 1
FROM categories WHERE slug = 'jeans';

-- ── Shoes (Tier-2: weight_kg × 14.0 kg CO₂e/kg) ─────────────────────────────
INSERT INTO products (name, brand, category_id, asin, url, image_url, co2e_kg, co2e_scope, co2e_source, co2e_confidence, weight_kg, origin_country, is_active)
SELECT 'Woodland Camel Leather Casual Shoes', 'Woodland',
  id, 'B07C65SX7K', 'https://www.amazon.in/dp/B07C65SX7K', 'https://m.media-amazon.com/images/P/B07C65SX7K.01._SL500_.jpg',
  ROUND(0.620 * 14.0, 2), 'lifecycle', 'estimated', 0.60, 0.620, 'IN', 1
FROM categories WHERE slug = 'shoes';

INSERT INTO products (name, brand, category_id, asin, url, image_url, co2e_kg, co2e_scope, co2e_source, co2e_confidence, weight_kg, origin_country, is_active)
SELECT 'Nike Revolution 6 Next Nature Running Shoes', 'Nike',
  id, 'B0979KRDV6', 'https://www.amazon.in/dp/B0979KRDV6', 'https://m.media-amazon.com/images/P/B0979KRDV6.01._SL500_.jpg',
  ROUND(0.280 * 14.0, 2), 'lifecycle', 'estimated', 0.60, 0.280, 'ID', 1
FROM categories WHERE slug = 'shoes';

-- ── Appliances (Tier-2: weight_kg × 6.0 kg CO₂e/kg) ─────────────────────────
INSERT INTO products (name, brand, category_id, asin, url, image_url, co2e_kg, co2e_scope, co2e_source, co2e_confidence, weight_kg, origin_country, is_active)
SELECT 'Havells Aspro 500W Mixer Grinder with 3 Jars', 'Havells',
  id, 'B09YQZL1HF', 'https://www.amazon.in/dp/B09YQZL1HF', 'https://m.media-amazon.com/images/P/B09YQZL1HF.01._SL500_.jpg',
  ROUND(3.50 * 6.0, 2), 'lifecycle', 'estimated', 0.60, 3.50, 'IN', 1
FROM categories WHERE slug = 'appliances';

INSERT INTO products (name, brand, category_id, asin, url, image_url, co2e_kg, co2e_scope, co2e_source, co2e_confidence, weight_kg, origin_country, is_active)
SELECT 'Orient Electric Aeroslim 1200mm BLDC Ceiling Fan', 'Orient Electric',
  id, 'B08VQXHNTM', 'https://www.amazon.in/dp/B08VQXHNTM', 'https://m.media-amazon.com/images/P/B08VQXHNTM.01._SL500_.jpg',
  ROUND(3.30 * 6.0, 2), 'lifecycle', 'estimated', 0.60, 3.30, 'IN', 1
FROM categories WHERE slug = 'appliances';

-- ── Furniture (Tier-2: weight_kg × 4.2 kg CO₂e/kg) ──────────────────────────
INSERT INTO products (name, brand, category_id, asin, url, image_url, co2e_kg, co2e_scope, co2e_source, co2e_confidence, weight_kg, origin_country, is_active)
SELECT 'Green Soul Montana Mid-Back Mesh Ergonomic Office Chair', 'Green Soul',
  id, 'B083T6XBZR', 'https://www.amazon.in/dp/B083T6XBZR', 'https://m.media-amazon.com/images/P/B083T6XBZR.01._SL500_.jpg',
  ROUND(14.0 * 4.2, 2), 'lifecycle', 'estimated', 0.55, 14.0, 'CN', 1
FROM categories WHERE slug = 'furniture';

INSERT INTO products (name, brand, category_id, asin, url, image_url, co2e_kg, co2e_scope, co2e_source, co2e_confidence, weight_kg, origin_country, is_active)
SELECT 'Nilkamal Novella Polypropylene Armchair', 'Nilkamal',
  id, 'B07WVNXLMK', 'https://www.amazon.in/dp/B07WVNXLMK', 'https://m.media-amazon.com/images/P/B07WVNXLMK.01._SL500_.jpg',
  ROUND(4.0 * 4.2, 2), 'lifecycle', 'estimated', 0.55, 4.0, 'IN', 1
FROM categories WHERE slug = 'furniture';
