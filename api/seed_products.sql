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

-- ── Headphones ────────────────────────────────────────────────────────────────
INSERT INTO products (name, brand, category_id, asin, url, image_url, co2e_kg, co2e_scope, co2e_source, co2e_confidence, weight_kg, origin_country, is_active)
SELECT 'Sony WH-1000XM5 Wireless Noise Cancelling Headphones', 'Sony',
  id, 'B09XS7JWHH', 'https://www.amazon.in/dp/B09XS7JWHH',
  'https://m.media-amazon.com/images/I/61vJtKbAssL._SL1500_.jpg',
  ROUND(0.250 * 22.0, 2), 'lifecycle', 'estimated', 0.65, 0.250, 'CN', 1
FROM categories WHERE slug = 'headphones';
