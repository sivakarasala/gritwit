ALTER TABLE wod_movements ADD COLUMN section_id UUID REFERENCES wod_sections(id) ON DELETE CASCADE;
ALTER TABLE wod_movements ADD COLUMN weight_kg_male REAL;
ALTER TABLE wod_movements ADD COLUMN weight_kg_female REAL;
ALTER TABLE wod_movements ADD COLUMN rep_scheme TEXT;

ALTER TABLE wod_movements DROP COLUMN wod_id;
ALTER TABLE wod_movements DROP COLUMN sets;
ALTER TABLE wod_movements DROP COLUMN reps;
ALTER TABLE wod_movements DROP COLUMN weight_kg;

CREATE INDEX idx_wod_movements_section ON wod_movements (section_id);
