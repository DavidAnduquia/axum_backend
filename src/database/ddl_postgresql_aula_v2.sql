-- =============================================
-- DDL PostgreSQL - Aula Virtual (VERSIÓN v2)
-- Archivo fuente: ddl_postgresql_aula_v2.sql
-- Schema objetivo: rustdema2
-- Nota: Este archivo ya incluye soft delete vía fecha_eliminacion donde aplica.
-- =============================================

-- rustdema2.areas_conocimiento definition

-- Drop table

-- DROP TABLE rustdema2.areas_conocimiento;

CREATE OR REPLACE FUNCTION actualizar_fecha_modificacion()
RETURNS TRIGGER AS $$
BEGIN
  NEW.fecha_actualizacion := NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TYPE rustdema2.tipo_evaluacion AS ENUM (
    'parcial',
    'final',
    'taller',
    'quiz'
);

CREATE TYPE rustdema2.tipo_pregunta AS ENUM (
    'opcion_multiple',
    'verdadero_falso',
    'respuesta_abierta'
);

CREATE TABLE rustdema2.areas_conocimiento (
	id serial4 NOT NULL,
	nombre varchar(100) NOT NULL,
	descripcion text NULL,
	color_etiqueta varchar(20) DEFAULT 'transparent'::character varying NOT NULL,
	estado bool DEFAULT true NOT NULL,
	fecha_creacion timestamptz DEFAULT now() NULL,
	fecha_actualizacion timestamptz DEFAULT now() NULL,
	CONSTRAINT areas_conocimiento_nombre_key UNIQUE (nombre),
	CONSTRAINT areas_conocimiento_pkey PRIMARY KEY (id)
);

-- Table Triggers

create trigger actualizar_areas_conocimiento_modtime before
update
    on
    rustdema2.areas_conocimiento for each row execute function rustdema2.actualizar_fecha_modificacion();


-- rustdema2.eventos_programados definition

-- Drop table

-- DROP TABLE rustdema2.eventos_programados;

CREATE TABLE rustdema2.eventos_programados (
	id serial4 NOT NULL,
	descripcion text NULL,
	fecha_ejecucion timestamptz DEFAULT now() NULL,
	estado varchar(20) DEFAULT 'pendiente'::character varying NULL,
	fecha_creacion timestamptz DEFAULT now() NULL,
	fecha_actualizacion timestamptz DEFAULT now() NULL,
	CONSTRAINT eventos_programados_pkey PRIMARY KEY (id)
);

-- Table Triggers

create trigger actualizar_eventos_programados_modtime before
update
    on
    rustdema2.eventos_programados for each row execute function rustdema2.actualizar_fecha_modificacion();


-- rustdema2.logros definition

-- Drop table

-- DROP TABLE rustdema2.logros;

CREATE TABLE rustdema2.logros (
	id serial4 NOT NULL,
	codigo varchar(50) NOT NULL,
	nombre varchar(100) NOT NULL,
	descripcion text NULL,
	url_icono text NULL,
	puntos_requeridos int4 NULL,
	nivel_insignia varchar(20) NULL,
	categoria varchar(50) NULL,
	fecha_creacion timestamptz DEFAULT now() NULL,
	CONSTRAINT logros_codigo_key UNIQUE (codigo),
	CONSTRAINT logros_nivel_insignia_check CHECK (((nivel_insignia)::text = ANY ((ARRAY['bronce'::character varying, 'plata'::character varying, 'oro'::character varying, 'platino'::character varying])::text[]))),
	CONSTRAINT logros_pkey PRIMARY KEY (id)
);


-- rustdema2.roles definition

-- Drop table

-- DROP TABLE rustdema2.roles;

CREATE TABLE rustdema2.roles (
	id serial4 NOT NULL,
	nombre varchar(50) NOT NULL,
	fecha_creacion timestamptz DEFAULT now() NULL,
	fecha_actualizacion timestamptz DEFAULT now() NULL,
	CONSTRAINT roles_nombre_key UNIQUE (nombre),
	CONSTRAINT roles_pkey PRIMARY KEY (id)
);

-- Table Triggers

create trigger actualizar_roles_modtime before
update
    on
    rustdema2.roles for each row execute function rustdema2.actualizar_fecha_modificacion();


-- rustdema2.usuarios definition

-- Drop table

-- DROP TABLE rustdema2.usuarios;

CREATE TABLE rustdema2.usuarios (
	id serial4 NOT NULL,
	nombre varchar(100) NOT NULL,
	documento_nit varchar(20) NOT NULL,
	correo varchar(100) NOT NULL,
	contrasena text NOT NULL,
	foto_url text NULL,
	rol_id int4 NULL,
	semestre int4 NULL,
	genero varchar(1) NOT NULL,
	fecha_nacimiento date NOT NULL,
	fecha_creacion timestamptz DEFAULT now() NULL,
	fecha_actualizacion timestamptz DEFAULT now() NULL,
	fecha_ultima_conexion timestamptz DEFAULT now() NULL,
	token_primer_ingreso timestamptz NULL,
	device_token varchar(255) NULL,
	estado bool DEFAULT true NULL,
	fecha_eliminacion timestamptz NULL,
	biografia varchar(600) NULL,
	CONSTRAINT chk_genero CHECK (((genero)::text = ANY ((ARRAY['M'::character varying, 'F'::character varying, 'O'::character varying])::text[]))),
	CONSTRAINT usuarios_correo_key UNIQUE (correo),
	CONSTRAINT usuarios_documento_nit_key UNIQUE (documento_nit),
	CONSTRAINT usuarios_pkey PRIMARY KEY (id),
	CONSTRAINT usuarios_rol_id_fkey FOREIGN KEY (rol_id) REFERENCES rustdema2.roles(id) ON DELETE SET NULL
);
CREATE INDEX idx_usuarios_activos ON rustdema2.usuarios USING btree (id) WHERE ((estado = true) AND (fecha_eliminacion IS NULL));
CREATE INDEX idx_usuarios_correo ON rustdema2.usuarios USING btree (correo);
CREATE INDEX idx_usuarios_rol_id ON rustdema2.usuarios USING btree (rol_id);

-- Table Triggers

create trigger actualizar_usuarios_modtime before
update
    on
    rustdema2.usuarios for each row execute function rustdema2.actualizar_fecha_modificacion();


-- rustdema2.cursos definition

-- Drop table

-- DROP TABLE rustdema2.cursos;

CREATE TABLE rustdema2.cursos (
	id serial4 NOT NULL,
	nombre varchar(100) NOT NULL,
	descripcion text NOT NULL,
	fecha_inicio date NOT NULL,
	fecha_fin date NOT NULL,
	prerequisito text NULL,
	coordinador_id int4 NOT NULL,
	creado_en timestamptz DEFAULT now() NULL,
	plantilla_base_id int4 NULL,
	semestre int4 NULL,
	periodo varchar(20) DEFAULT ''::character varying NOT NULL,
	anio_pensum int4 NOT NULL,
	area_conocimiento_id int4 NOT NULL,
	fecha_eliminacion timestamptz NULL,
	fecha_actualizacion timestamptz DEFAULT now() NULL,
	CONSTRAINT chk_fechas_curso CHECK ((fecha_fin >= fecha_inicio)),
	CONSTRAINT cursos_pkey PRIMARY KEY (id),
	CONSTRAINT cursos_area_conocimiento_id_fkey FOREIGN KEY (area_conocimiento_id) REFERENCES rustdema2.areas_conocimiento(id) ON DELETE CASCADE,
	CONSTRAINT cursos_coordinador_id_fkey FOREIGN KEY (coordinador_id) REFERENCES rustdema2.usuarios(id) ON DELETE CASCADE,
	CONSTRAINT cursos_plantilla_base_id_fkey FOREIGN KEY (plantilla_base_id) REFERENCES rustdema2.cursos(id) ON DELETE SET NULL
);
CREATE INDEX idx_cursos_activos ON rustdema2.cursos USING btree (id) WHERE (fecha_eliminacion IS NULL);
CREATE INDEX idx_cursos_area ON rustdema2.cursos USING btree (area_conocimiento_id);
CREATE INDEX idx_cursos_coordinador ON rustdema2.cursos USING btree (coordinador_id);

-- Table Triggers

create trigger actualizar_cursos_modtime before
update
    on
    rustdema2.cursos for each row execute function rustdema2.actualizar_fecha_modificacion();


-- rustdema2.evaluaciones definition

-- Drop table

-- DROP TABLE rustdema2.evaluaciones;

CREATE TABLE rustdema2.evaluaciones (
	id serial4 NOT NULL,
	id_curso int4 NOT NULL,
	tipo rustdema2."tipo_evaluacion" NOT NULL,
	titulo varchar(200) NOT NULL,
	instrucciones text NULL,
	configuracion jsonb DEFAULT '{}'::jsonb NOT NULL,
	peso numeric(5, 2) DEFAULT 1.0 NULL,
	puntaje_maximo numeric(5, 2) DEFAULT 100.0 NULL,
	fecha_disponible_desde timestamptz NULL,
	fecha_disponible_hasta timestamptz NULL,
	fecha_limite_entrega timestamptz NULL,
	estado varchar(20) DEFAULT 'borrador'::character varying NULL,
	creado_por int4 NULL,
	fecha_creacion timestamptz DEFAULT now() NULL,
	fecha_actualizacion timestamptz DEFAULT now() NULL,
	fecha_eliminacion timestamptz NULL,
	CONSTRAINT evaluaciones_check CHECK ((fecha_disponible_hasta >= fecha_disponible_desde)),
	CONSTRAINT evaluaciones_estado_check CHECK (((estado)::text = ANY ((ARRAY['borrador'::character varying, 'publicado'::character varying, 'archivado'::character varying])::text[]))),
	CONSTRAINT evaluaciones_pkey PRIMARY KEY (id),
	CONSTRAINT evaluaciones_creado_por_fkey FOREIGN KEY (creado_por) REFERENCES rustdema2.usuarios(id),
	CONSTRAINT evaluaciones_id_curso_fkey FOREIGN KEY (id_curso) REFERENCES rustdema2.cursos(id) ON DELETE CASCADE
);
CREATE INDEX idx_evaluaciones_curso ON rustdema2.evaluaciones USING btree (id_curso, fecha_limite_entrega);
CREATE INDEX idx_evaluaciones_tipo ON rustdema2.evaluaciones USING btree (tipo, estado);

-- Table Triggers

create trigger actualizar_evaluaciones_modtime before
update
    on
    rustdema2.evaluaciones for each row execute function rustdema2.actualizar_fecha_modificacion();


-- rustdema2.eventos_puntos definition

-- Drop table

-- DROP TABLE rustdema2.eventos_puntos;

CREATE TABLE rustdema2.eventos_puntos (
	id serial4 NOT NULL,
	id_estudiante int4 NOT NULL,
	id_curso int4 NOT NULL,
	tipo_evento varchar(50) NOT NULL,
	puntos int4 NOT NULL,
	descripcion text NULL,
	id_referencia int4 NULL,
	tipo_referencia varchar(50) NULL,
	fecha_creacion timestamptz DEFAULT now() NULL,
	CONSTRAINT eventos_puntos_pkey PRIMARY KEY (id),
	CONSTRAINT eventos_puntos_id_curso_fkey FOREIGN KEY (id_curso) REFERENCES rustdema2.cursos(id) ON DELETE CASCADE,
	CONSTRAINT eventos_puntos_id_estudiante_fkey FOREIGN KEY (id_estudiante) REFERENCES rustdema2.usuarios(id) ON DELETE CASCADE
);
CREATE INDEX idx_eventos_puntos_estudiante ON rustdema2.eventos_puntos USING btree (id_estudiante, id_curso, fecha_creacion DESC);


-- rustdema2.examenes definition

-- Drop table

-- DROP TABLE rustdema2.examenes;

CREATE TABLE rustdema2.examenes (
	id serial4 NOT NULL,
	curso_id int4 NULL,
	nombre varchar(200) NOT NULL,
	descripcion text NULL,
	fecha_inicio timestamptz NULL,
	fecha_fin timestamptz NULL,
	fecha_eliminacion timestamptz NULL,
	CONSTRAINT examenes_pkey PRIMARY KEY (id),
	CONSTRAINT examenes_curso_id_fkey FOREIGN KEY (curso_id) REFERENCES rustdema2.cursos(id) ON DELETE CASCADE
);


-- rustdema2.historial_cursos_estudiantes definition

-- Drop table

-- DROP TABLE rustdema2.historial_cursos_estudiantes;

CREATE TABLE rustdema2.historial_cursos_estudiantes (
	id serial4 NOT NULL,
	curso_id int4 NOT NULL,
	estudiante_id int4 NOT NULL,
	fecha_inscripcion timestamptz DEFAULT now() NULL,
	estado varchar(30) DEFAULT 'en_progreso'::character varying NOT NULL,
	calificacion_final float8 NULL,
	aprobado bool DEFAULT false NOT NULL,
	fecha_creacion timestamptz DEFAULT now() NULL,
	fecha_actualizacion timestamptz DEFAULT now() NULL,
	CONSTRAINT historial_cursos_estudiantes_pkey PRIMARY KEY (id),
	CONSTRAINT uq_historial_curso_estudiante UNIQUE (curso_id, estudiante_id),
	CONSTRAINT historial_cursos_estudiantes_curso_id_fkey FOREIGN KEY (curso_id) REFERENCES rustdema2.cursos(id) ON DELETE CASCADE,
	CONSTRAINT historial_cursos_estudiantes_estudiante_id_fkey FOREIGN KEY (estudiante_id) REFERENCES rustdema2.usuarios(id) ON DELETE CASCADE
);
CREATE INDEX idx_historial_estudiante_estado ON rustdema2.historial_cursos_estudiantes USING btree (estudiante_id, estado);


-- rustdema2.logros_estudiante definition

-- Drop table

-- DROP TABLE rustdema2.logros_estudiante;

CREATE TABLE rustdema2.logros_estudiante (
	id_estudiante int4 NOT NULL,
	id_logro int4 NOT NULL,
	id_curso int4 NOT NULL,
	fecha_obtencion timestamptz DEFAULT now() NULL,
	CONSTRAINT logros_estudiante_pkey PRIMARY KEY (id_estudiante, id_logro, id_curso),
	CONSTRAINT logros_estudiante_id_curso_fkey FOREIGN KEY (id_curso) REFERENCES rustdema2.cursos(id) ON DELETE CASCADE,
	CONSTRAINT logros_estudiante_id_estudiante_fkey FOREIGN KEY (id_estudiante) REFERENCES rustdema2.usuarios(id) ON DELETE CASCADE,
	CONSTRAINT logros_estudiante_id_logro_fkey FOREIGN KEY (id_logro) REFERENCES rustdema2.logros(id) ON DELETE CASCADE
);


-- rustdema2.modulos definition

-- Drop table

-- DROP TABLE rustdema2.modulos;

CREATE TABLE rustdema2.modulos (
	id serial4 NOT NULL,
	curso_id int4 NOT NULL,
	nombre varchar(200) NOT NULL,
	descripcion text NULL,
	orden int4 DEFAULT 0 NOT NULL,
	tipo varchar(50) DEFAULT 'estructura_contenido'::character varying NOT NULL,
	visible bool DEFAULT true NOT NULL,
	fecha_inicio timestamptz NULL,
	fecha_fin timestamptz NULL,
	duracion_estimada int4 NULL,
	obligatorio bool DEFAULT true NOT NULL,
	fecha_creacion timestamptz DEFAULT now() NULL,
	fecha_actualizacion timestamptz DEFAULT now() NULL,
	fecha_eliminacion timestamptz NULL,
	CONSTRAINT modulos_pkey PRIMARY KEY (id),
	CONSTRAINT modulos_curso_id_fkey FOREIGN KEY (curso_id) REFERENCES rustdema2.cursos(id) ON DELETE CASCADE
);
CREATE INDEX idx_modulos_activos ON rustdema2.modulos USING btree (id) WHERE (fecha_eliminacion IS NULL);
CREATE INDEX idx_modulos_curso ON rustdema2.modulos USING btree (curso_id);
CREATE INDEX idx_modulos_tipo ON rustdema2.modulos USING btree (tipo);

-- Table Triggers

create trigger actualizar_modulos_modtime before
update
    on
    rustdema2.modulos for each row execute function rustdema2.actualizar_fecha_modificacion();


-- rustdema2.notificaciones definition

-- Drop table

-- DROP TABLE rustdema2.notificaciones;

CREATE TABLE rustdema2.notificaciones (
	id serial4 NOT NULL,
	usuario_id int4 NOT NULL,
	titulo varchar(255) NOT NULL,
	mensaje text NOT NULL,
	tipo varchar(50) NOT NULL,
	leida bool DEFAULT false NULL,
	enlace varchar(500) NULL,
	datos_adicionales jsonb NULL,
	fecha_creacion timestamptz DEFAULT now() NULL,
	fecha_actualizacion timestamptz DEFAULT now() NULL,
	CONSTRAINT notificaciones_pkey PRIMARY KEY (id),
	CONSTRAINT notificaciones_usuario_id_fkey FOREIGN KEY (usuario_id) REFERENCES rustdema2.usuarios(id) ON DELETE CASCADE
);
CREATE INDEX idx_notificaciones_leida ON rustdema2.notificaciones USING btree (leida);
CREATE INDEX idx_notificaciones_tipo ON rustdema2.notificaciones USING btree (tipo);
CREATE INDEX idx_notificaciones_usuario_id ON rustdema2.notificaciones USING btree (usuario_id);

-- Table Triggers

create trigger actualizar_notificaciones_modtime before
update
    on
    rustdema2.notificaciones for each row execute function rustdema2.actualizar_fecha_modificacion();


-- rustdema2.personalizacion_examen definition

-- Drop table

-- DROP TABLE rustdema2.personalizacion_examen;

CREATE TABLE rustdema2.personalizacion_examen (
	id serial4 NOT NULL,
	examen_id int4 NULL,
	estilos jsonb NULL,
	orden_componentes jsonb NULL,
	privacidad_componentes jsonb NULL,
	CONSTRAINT personalizacion_examen_pkey PRIMARY KEY (id),
	CONSTRAINT uq_personalizacion_examen UNIQUE (examen_id),
	CONSTRAINT personalizacion_examen_examen_id_fkey FOREIGN KEY (examen_id) REFERENCES rustdema2.examenes(id) ON DELETE CASCADE
);


-- rustdema2.personalizacion_modulo definition

-- Drop table

-- DROP TABLE rustdema2.personalizacion_modulo;

CREATE TABLE rustdema2.personalizacion_modulo (
	id serial4 NOT NULL,
	modulo_id int4 NULL,
	estilos jsonb NULL,
	orden_componentes jsonb NULL,
	privacidad_componentes jsonb NULL,
	CONSTRAINT personalizacion_modulo_pkey PRIMARY KEY (id),
	CONSTRAINT uq_personalizacion_modulo UNIQUE (modulo_id),
	CONSTRAINT personalizacion_modulo_modulo_id_fkey FOREIGN KEY (modulo_id) REFERENCES rustdema2.modulos(id) ON DELETE CASCADE
);


-- rustdema2.plantillas_certificado definition

-- Drop table

-- DROP TABLE rustdema2.plantillas_certificado;

CREATE TABLE rustdema2.plantillas_certificado (
	id serial4 NOT NULL,
	nombre varchar(100) NOT NULL,
	descripcion text NULL,
	plantilla_html text NOT NULL,
	url_imagen_fondo text NULL,
	configuracion jsonb DEFAULT '{}'::jsonb NULL,
	activa bool DEFAULT true NULL,
	creado_por int4 NULL,
	fecha_creacion timestamptz DEFAULT now() NULL,
	fecha_actualizacion timestamptz DEFAULT now() NULL,
	CONSTRAINT plantillas_certificado_pkey PRIMARY KEY (id),
	CONSTRAINT plantillas_certificado_creado_por_fkey FOREIGN KEY (creado_por) REFERENCES rustdema2.usuarios(id)
);

-- Table Triggers

create trigger actualizar_plantillas_certificado_modtime before
update
    on
    rustdema2.plantillas_certificado for each row execute function rustdema2.actualizar_fecha_modificacion();


-- rustdema2.plantillas_cursos definition

-- Drop table

-- DROP TABLE rustdema2.plantillas_cursos;

CREATE TABLE rustdema2.plantillas_cursos (
	id serial4 NOT NULL,
	nombre varchar(150) NOT NULL,
	descripcion text NULL,
	activa bool DEFAULT true NOT NULL,
	curso_id int4 NULL,
	fecha_creacion timestamptz DEFAULT now() NULL,
	fecha_actualizacion timestamptz DEFAULT now() NULL,
	CONSTRAINT plantillas_cursos_pkey PRIMARY KEY (id),
	CONSTRAINT plantillas_cursos_curso_id_fkey FOREIGN KEY (curso_id) REFERENCES rustdema2.cursos(id) ON DELETE CASCADE
);

-- Table Triggers

create trigger actualizar_plantillas_cursos_modtime before
update
    on
    rustdema2.plantillas_cursos for each row execute function rustdema2.actualizar_fecha_modificacion();


-- rustdema2.portafolios definition

-- Drop table

-- DROP TABLE rustdema2.portafolios;

CREATE TABLE rustdema2.portafolios (
	id serial4 NOT NULL,
	curso_id int4 NULL,
	nombre varchar(200) NOT NULL,
	descripcion text NULL,
	CONSTRAINT portafolios_pkey PRIMARY KEY (id),
	CONSTRAINT portafolios_curso_id_fkey FOREIGN KEY (curso_id) REFERENCES rustdema2.cursos(id) ON DELETE CASCADE
);


-- rustdema2.preguntas_examen definition

-- Drop table

-- DROP TABLE rustdema2.preguntas_examen;

CREATE TABLE rustdema2.preguntas_examen (
	id serial4 NOT NULL,
	examen_id int4 NULL,
	enunciado text NOT NULL,
	tipo varchar(20) NOT NULL,
	opciones jsonb NULL,
	respuesta_correcta text NULL,
	fecha_inicio_pregunta timestamptz NULL,
	fecha_fin_pregunta timestamptz NULL,
	porcentaje numeric(5, 2) DEFAULT 0 NULL,
	CONSTRAINT preguntas_examen_pkey PRIMARY KEY (id),
	CONSTRAINT preguntas_examen_tipo_check CHECK (((tipo)::text = ANY ((ARRAY['abierta'::character varying, 'seleccion_unica'::character varying, 'seleccion_multiple'::character varying])::text[]))),
	CONSTRAINT preguntas_examen_examen_id_fkey FOREIGN KEY (examen_id) REFERENCES rustdema2.examenes(id) ON DELETE CASCADE
);


-- rustdema2.profesores_curso definition

-- Drop table

-- DROP TABLE rustdema2.profesores_curso;

CREATE TABLE rustdema2.profesores_curso (
	id serial4 NOT NULL,
	profesor_id int4 NULL,
	curso_id int4 NULL,
	fecha_asignacion timestamptz DEFAULT now() NULL,
	CONSTRAINT profesores_curso_pkey PRIMARY KEY (id),
	CONSTRAINT uq_profesor_curso UNIQUE (profesor_id, curso_id),
	CONSTRAINT profesores_curso_curso_id_fkey FOREIGN KEY (curso_id) REFERENCES rustdema2.cursos(id) ON DELETE CASCADE,
	CONSTRAINT profesores_curso_profesor_id_fkey FOREIGN KEY (profesor_id) REFERENCES rustdema2.usuarios(id) ON DELETE CASCADE
);


-- rustdema2.puntos_estudiante definition

-- Drop table

-- DROP TABLE rustdema2.puntos_estudiante;

CREATE TABLE rustdema2.puntos_estudiante (
	id_estudiante int4 NOT NULL,
	id_curso int4 NOT NULL,
	puntos_totales int4 DEFAULT 0 NULL,
	nivel int4 DEFAULT 1 NULL,
	racha_actual_dias int4 DEFAULT 0 NULL,
	racha_maxima_dias int4 DEFAULT 0 NULL,
	ultima_actividad date NULL,
	fecha_creacion timestamptz DEFAULT now() NULL,
	fecha_actualizacion timestamptz DEFAULT now() NULL,
	CONSTRAINT puntos_estudiante_pkey PRIMARY KEY (id_estudiante, id_curso),
	CONSTRAINT puntos_estudiante_id_curso_fkey FOREIGN KEY (id_curso) REFERENCES rustdema2.cursos(id) ON DELETE CASCADE,
	CONSTRAINT puntos_estudiante_id_estudiante_fkey FOREIGN KEY (id_estudiante) REFERENCES rustdema2.usuarios(id) ON DELETE CASCADE
);
CREATE INDEX idx_puntos_estudiante_curso ON rustdema2.puntos_estudiante USING btree (id_estudiante, id_curso);

-- Table Triggers

create trigger actualizar_puntos_estudiante_modtime before
update
    on
    rustdema2.puntos_estudiante for each row execute function rustdema2.actualizar_fecha_modificacion();


-- rustdema2.sesiones_curso definition

-- Drop table

-- DROP TABLE rustdema2.sesiones_curso;

CREATE TABLE rustdema2.sesiones_curso (
	id serial4 NOT NULL,
	curso_id int4 NULL,
	nombre_sesion varchar(200) NULL,
	descripcion text NULL,
	fecha_inicio timestamptz NULL,
	fecha_fin timestamptz NULL,
	orden int4 NOT NULL,
	CONSTRAINT sesiones_curso_pkey PRIMARY KEY (id),
	CONSTRAINT sesiones_curso_curso_id_fkey FOREIGN KEY (curso_id) REFERENCES rustdema2.cursos(id) ON DELETE CASCADE
);


-- rustdema2.temas definition

-- Drop table

-- DROP TABLE rustdema2.temas;

CREATE TABLE rustdema2.temas (
	id serial4 NOT NULL,
	modulo_id int4 NOT NULL,
	nombre varchar(200) NOT NULL,
	descripcion text NULL,
	orden int4 DEFAULT 0 NOT NULL,
	visible bool DEFAULT true NOT NULL,
	fecha_creacion timestamptz DEFAULT now() NULL,
	fecha_actualizacion timestamptz DEFAULT now() NULL,
	CONSTRAINT temas_pkey PRIMARY KEY (id),
	CONSTRAINT temas_modulo_id_fkey FOREIGN KEY (modulo_id) REFERENCES rustdema2.modulos(id) ON DELETE CASCADE
);
CREATE INDEX idx_temas_modulo ON rustdema2.temas USING btree (modulo_id);

-- Table Triggers

create trigger actualizar_temas_modtime before
update
    on
    rustdema2.temas for each row execute function rustdema2.actualizar_fecha_modificacion();


-- rustdema2.unidades definition

-- Drop table

-- DROP TABLE rustdema2.unidades;

CREATE TABLE rustdema2.unidades (
	id serial4 NOT NULL,
	tema_id int4 NOT NULL,
	nombre varchar(200) NOT NULL,
	descripcion text NULL,
	orden int4 DEFAULT 0 NOT NULL,
	visible bool DEFAULT true NOT NULL,
	fecha_creacion timestamptz DEFAULT now() NULL,
	fecha_actualizacion timestamptz DEFAULT now() NULL,
	CONSTRAINT unidades_pkey PRIMARY KEY (id),
	CONSTRAINT unidades_tema_id_fkey FOREIGN KEY (tema_id) REFERENCES rustdema2.temas(id) ON DELETE CASCADE
);
CREATE INDEX idx_unidades_tema ON rustdema2.unidades USING btree (tema_id);

-- Table Triggers

create trigger actualizar_unidades_modtime before
update
    on
    rustdema2.unidades for each row execute function rustdema2.actualizar_fecha_modificacion();


-- rustdema2.webinars definition

-- Drop table

-- DROP TABLE rustdema2.webinars;

CREATE TABLE rustdema2.webinars (
	id serial4 NOT NULL,
	curso_id int4 NULL,
	titulo varchar(200) NOT NULL,
	descripcion text NULL,
	progreso int4 DEFAULT 0 NOT NULL,
	estado varchar(50) DEFAULT 'no_iniciado'::character varying NOT NULL,
	duracion varchar(50) NULL,
	modulos int4 DEFAULT 1 NOT NULL,
	fecha_creacion timestamptz DEFAULT now() NULL,
	fecha_actualizacion timestamptz DEFAULT now() NULL,
	CONSTRAINT webinars_estado_check CHECK (((estado)::text = ANY ((ARRAY['no_iniciado'::character varying, 'en_progreso'::character varying, 'completado'::character varying])::text[]))),
	CONSTRAINT webinars_pkey PRIMARY KEY (id),
	CONSTRAINT webinars_progreso_check CHECK (((progreso >= 0) AND (progreso <= 100))),
	CONSTRAINT webinars_curso_id_fkey FOREIGN KEY (curso_id) REFERENCES rustdema2.cursos(id) ON DELETE CASCADE
);
CREATE INDEX idx_webinars_curso ON rustdema2.webinars USING btree (curso_id);

-- Table Triggers

create trigger actualizar_webinars_modtime before
update
    on
    rustdema2.webinars for each row execute function rustdema2.actualizar_fecha_modificacion();


-- rustdema2.actividades definition

-- Drop table

-- DROP TABLE rustdema2.actividades;

CREATE TABLE rustdema2.actividades (
	id serial4 NOT NULL,
	curso_id int4 NULL,
	profesor_id int4 NULL,
	nombre varchar(200) NOT NULL,
	descripcion text NULL,
	privacidad varchar(20) NULL,
	fecha_inicio timestamptz NULL,
	fecha_fin timestamptz NULL,
	tipo_actividad varchar(50) NOT NULL,
	fecha_eliminacion timestamptz NULL,
	CONSTRAINT actividades_pkey PRIMARY KEY (id),
	CONSTRAINT actividades_curso_id_fkey FOREIGN KEY (curso_id) REFERENCES rustdema2.cursos(id) ON DELETE CASCADE,
	CONSTRAINT actividades_profesor_id_fkey FOREIGN KEY (profesor_id) REFERENCES rustdema2.usuarios(id) ON DELETE CASCADE
);
CREATE INDEX idx_actividades_activas ON rustdema2.actividades USING btree (id) WHERE (fecha_eliminacion IS NULL);
CREATE INDEX idx_actividades_curso ON rustdema2.actividades USING btree (curso_id);
CREATE INDEX idx_actividades_tipo_fecha ON rustdema2.actividades USING btree (tipo_actividad, fecha_fin);


-- rustdema2.actividades_entrega definition

-- Drop table

-- DROP TABLE rustdema2.actividades_entrega;

CREATE TABLE rustdema2.actividades_entrega (
	id serial4 NOT NULL,
	unidad_id int4 NULL,
	nombre varchar(200) NOT NULL,
	descripcion text NULL,
	fecha_limite timestamptz NOT NULL,
	tipo_actividad varchar(50) NOT NULL,
	activo bool DEFAULT true NOT NULL,
	fecha_creacion timestamptz DEFAULT now() NULL,
	fecha_actualizacion timestamptz DEFAULT now() NULL,
	CONSTRAINT actividades_entrega_pkey PRIMARY KEY (id),
	CONSTRAINT actividades_entrega_tipo_actividad_check CHECK (((tipo_actividad)::text = ANY ((ARRAY['entrega_obligatoria'::character varying, 'entrega_opcional'::character varying])::text[]))),
	CONSTRAINT actividades_entrega_unidad_id_fkey FOREIGN KEY (unidad_id) REFERENCES rustdema2.unidades(id) ON DELETE CASCADE
);
CREATE INDEX idx_actividades_entrega_unidad ON rustdema2.actividades_entrega USING btree (unidad_id);

-- Table Triggers

create trigger actualizar_actividades_entrega_modtime before
update
    on
    rustdema2.actividades_entrega for each row execute function rustdema2.actualizar_fecha_modificacion();


-- rustdema2.banco_preguntas definition

-- Drop table

-- DROP TABLE rustdema2.banco_preguntas;

CREATE TABLE rustdema2.banco_preguntas (
	id serial4 NOT NULL,
	id_curso int4 NULL,
	tipo rustdema2."tipo_pregunta" NOT NULL,
	contenido text NOT NULL,
	opciones jsonb DEFAULT '[]'::jsonb NULL,
	respuesta_correcta jsonb NULL,
	puntaje numeric(5, 2) DEFAULT 1.0 NULL,
	dificultad varchar(20) NULL,
	etiquetas _text DEFAULT '{}'::text[] NULL,
	explicacion text NULL,
	fecha_creacion timestamptz DEFAULT now() NULL,
	fecha_actualizacion timestamptz DEFAULT now() NULL,
	fecha_eliminacion timestamptz NULL,
	CONSTRAINT banco_preguntas_dificultad_check CHECK (((dificultad)::text = ANY ((ARRAY['facil'::character varying, 'medio'::character varying, 'dificil'::character varying])::text[]))),
	CONSTRAINT banco_preguntas_pkey PRIMARY KEY (id),
	CONSTRAINT banco_preguntas_id_curso_fkey FOREIGN KEY (id_curso) REFERENCES rustdema2.cursos(id) ON DELETE CASCADE
);
CREATE INDEX idx_preguntas_curso ON rustdema2.banco_preguntas USING btree (id_curso);
CREATE INDEX idx_preguntas_etiquetas ON rustdema2.banco_preguntas USING gin (etiquetas);

-- Table Triggers

create trigger actualizar_banco_preguntas_modtime before
update
    on
    rustdema2.banco_preguntas for each row execute function rustdema2.actualizar_fecha_modificacion();


-- rustdema2.calificaciones definition

-- Drop table

-- DROP TABLE rustdema2.calificaciones;

CREATE TABLE rustdema2.calificaciones (
	id serial4 NOT NULL,
	actividad_id int4 NULL,
	estudiante_id int4 NULL,
	calificacion numeric(5, 2) NULL,
	fecha_registro timestamptz DEFAULT now() NULL,
	CONSTRAINT calificaciones_pkey PRIMARY KEY (id),
	CONSTRAINT uq_actividad_estudiante UNIQUE (actividad_id, estudiante_id),
	CONSTRAINT calificaciones_actividad_id_fkey FOREIGN KEY (actividad_id) REFERENCES rustdema2.actividades(id) ON DELETE CASCADE,
	CONSTRAINT calificaciones_estudiante_id_fkey FOREIGN KEY (estudiante_id) REFERENCES rustdema2.usuarios(id) ON DELETE CASCADE
);
CREATE INDEX idx_calificaciones_actividad ON rustdema2.calificaciones USING btree (actividad_id);
CREATE INDEX idx_calificaciones_composite ON rustdema2.calificaciones USING btree (actividad_id, estudiante_id);
CREATE INDEX idx_calificaciones_estudiante ON rustdema2.calificaciones USING btree (estudiante_id);


-- rustdema2.calificaciones_evaluacion definition

-- Drop table

-- DROP TABLE rustdema2.calificaciones_evaluacion;

CREATE TABLE rustdema2.calificaciones_evaluacion (
	id serial4 NOT NULL,
	id_evaluacion int4 NOT NULL,
	id_estudiante int4 NOT NULL,
	puntaje_obtenido numeric(5, 2) NULL,
	puntaje_maximo numeric(5, 2) NOT NULL,
	retroalimentacion text NULL,
	calificado_por int4 NULL,
	fecha_calificacion timestamptz NULL,
	estado varchar(20) DEFAULT 'pendiente'::character varying NULL,
	fecha_creacion timestamptz DEFAULT now() NULL,
	fecha_actualizacion timestamptz DEFAULT now() NULL,
	CONSTRAINT calificaciones_evaluacion_estado_check CHECK (((estado)::text = ANY ((ARRAY['pendiente'::character varying, 'calificado'::character varying, 'en_progreso'::character varying])::text[]))),
	CONSTRAINT calificaciones_evaluacion_id_evaluacion_id_estudiante_key UNIQUE (id_evaluacion, id_estudiante),
	CONSTRAINT calificaciones_evaluacion_pkey PRIMARY KEY (id),
	CONSTRAINT calificaciones_evaluacion_calificado_por_fkey FOREIGN KEY (calificado_por) REFERENCES rustdema2.usuarios(id),
	CONSTRAINT calificaciones_evaluacion_id_estudiante_fkey FOREIGN KEY (id_estudiante) REFERENCES rustdema2.usuarios(id) ON DELETE CASCADE,
	CONSTRAINT calificaciones_evaluacion_id_evaluacion_fkey FOREIGN KEY (id_evaluacion) REFERENCES rustdema2.evaluaciones(id) ON DELETE CASCADE
);
CREATE INDEX idx_calificaciones_evaluacion_estado ON rustdema2.calificaciones_evaluacion USING btree (id_evaluacion, estado);
CREATE INDEX idx_calificaciones_evaluacion_estudiante ON rustdema2.calificaciones_evaluacion USING btree (id_estudiante, id_evaluacion);

-- Table Triggers

create trigger actualizar_calificaciones_evaluacion_modtime before
update
    on
    rustdema2.calificaciones_evaluacion for each row execute function rustdema2.actualizar_fecha_modificacion();


-- rustdema2.certificados definition

-- Drop table

-- DROP TABLE rustdema2.certificados;

CREATE TABLE rustdema2.certificados (
	id uuid DEFAULT gen_random_uuid() NOT NULL,
	id_estudiante int4 NOT NULL,
	id_curso int4 NOT NULL,
	id_plantilla int4 NULL,
	numero_certificado varchar(50) NOT NULL,
	fecha_emision timestamptz DEFAULT now() NULL,
	fecha_expiracion timestamptz NULL,
	metadatos jsonb DEFAULT '{}'::jsonb NULL,
	codigo_verificacion varchar(50) NOT NULL,
	url_pdf text NULL,
	estado varchar(20) DEFAULT 'activo'::character varying NULL,
	fecha_creacion timestamptz DEFAULT now() NULL,
	fecha_actualizacion timestamptz DEFAULT now() NULL,
	CONSTRAINT certificados_codigo_verificacion_key UNIQUE (codigo_verificacion),
	CONSTRAINT certificados_estado_check CHECK (((estado)::text = ANY ((ARRAY['activo'::character varying, 'revocado'::character varying, 'expirado'::character varying])::text[]))),
	CONSTRAINT certificados_numero_certificado_key UNIQUE (numero_certificado),
	CONSTRAINT certificados_pkey PRIMARY KEY (id),
	CONSTRAINT certificados_id_curso_fkey FOREIGN KEY (id_curso) REFERENCES rustdema2.cursos(id) ON DELETE CASCADE,
	CONSTRAINT certificados_id_estudiante_fkey FOREIGN KEY (id_estudiante) REFERENCES rustdema2.usuarios(id) ON DELETE CASCADE,
	CONSTRAINT certificados_id_plantilla_fkey FOREIGN KEY (id_plantilla) REFERENCES rustdema2.plantillas_certificado(id) ON DELETE SET NULL
);
CREATE INDEX idx_certificados_estudiante ON rustdema2.certificados USING btree (id_estudiante, id_curso);
CREATE INDEX idx_certificados_verificacion ON rustdema2.certificados USING btree (codigo_verificacion);

-- Table Triggers

create trigger actualizar_certificados_modtime before
update
    on
    rustdema2.certificados for each row execute function rustdema2.actualizar_fecha_modificacion();


-- rustdema2.contenido_transversal definition

-- Drop table

-- DROP TABLE rustdema2.contenido_transversal;

CREATE TABLE rustdema2.contenido_transversal (
	id serial4 NOT NULL,
	curso_id int4 NULL,
	origen_tipo varchar(20) NOT NULL,
	origen_id int4 NOT NULL,
	profesor_id int4 NULL,
	tipo_contenido varchar(50) NOT NULL,
	ruta_archivo text NULL,
	enlace_video text NULL,
	fecha_subida timestamptz DEFAULT now() NULL,
	privacidad varchar(20) NULL,
	descripcion text NULL,
	CONSTRAINT contenido_transversal_origen_tipo_check CHECK (((origen_tipo)::text = ANY ((ARRAY['curso'::character varying, 'examen'::character varying, 'modulo'::character varying, 'portafolio'::character varying])::text[]))),
	CONSTRAINT contenido_transversal_pkey PRIMARY KEY (id),
	CONSTRAINT contenido_transversal_curso_id_fkey FOREIGN KEY (curso_id) REFERENCES rustdema2.cursos(id) ON DELETE CASCADE,
	CONSTRAINT contenido_transversal_profesor_id_fkey FOREIGN KEY (profesor_id) REFERENCES rustdema2.usuarios(id) ON DELETE CASCADE
);
CREATE INDEX idx_contenido_transversal_curso_origen ON rustdema2.contenido_transversal USING btree (curso_id, origen_tipo);


-- rustdema2.entregas definition

-- Drop table

-- DROP TABLE rustdema2.entregas;

CREATE TABLE rustdema2.entregas (
	id serial4 NOT NULL,
	actividad_entrega_id int4 NULL,
	estudiante_id int4 NULL,
	documento_nombre varchar(255) NOT NULL,
	documento_tipo varchar(100) NOT NULL,
	documento_tamanio int8 NOT NULL,
	documento_url text NOT NULL,
	fecha_entrega timestamptz DEFAULT now() NOT NULL,
	calificacion float4 NULL,
	comentario_profesor text NULL,
	fecha_calificacion timestamptz NULL,
	estado varchar(50) DEFAULT 'pendiente'::character varying NOT NULL,
	fecha_creacion timestamptz DEFAULT now() NULL,
	fecha_actualizacion timestamptz DEFAULT now() NULL,
	CONSTRAINT entregas_estado_check CHECK (((estado)::text = ANY ((ARRAY['pendiente'::character varying, 'calificado'::character varying, 'rechazado'::character varying])::text[]))),
	CONSTRAINT entregas_pkey PRIMARY KEY (id),
	CONSTRAINT entregas_actividad_entrega_id_fkey FOREIGN KEY (actividad_entrega_id) REFERENCES rustdema2.actividades_entrega(id) ON DELETE CASCADE,
	CONSTRAINT entregas_estudiante_id_fkey FOREIGN KEY (estudiante_id) REFERENCES rustdema2.usuarios(id) ON DELETE CASCADE
);
CREATE INDEX idx_entregas_actividad ON rustdema2.entregas USING btree (actividad_entrega_id);
CREATE INDEX idx_entregas_estudiante ON rustdema2.entregas USING btree (estudiante_id);

-- Table Triggers

create trigger actualizar_entregas_modtime before
update
    on
    rustdema2.entregas for each row execute function rustdema2.actualizar_fecha_modificacion();


-- rustdema2.evaluaciones_sesion definition

-- Drop table

-- DROP TABLE rustdema2.evaluaciones_sesion;

CREATE TABLE rustdema2.evaluaciones_sesion (
	id serial4 NOT NULL,
	sesion_id int4 NULL,
	nombre_evaluacion varchar(200) NULL,
	tipo_actividad varchar(50) NOT NULL,
	fecha_inicio timestamptz NULL,
	fecha_fin timestamptz NULL,
	CONSTRAINT evaluaciones_sesion_pkey PRIMARY KEY (id),
	CONSTRAINT evaluaciones_sesion_sesion_id_fkey FOREIGN KEY (sesion_id) REFERENCES rustdema2.sesiones_curso(id) ON DELETE CASCADE
);


-- rustdema2.historial_curso_actividad definition

-- Drop table

-- DROP TABLE rustdema2.historial_curso_actividad;

CREATE TABLE rustdema2.historial_curso_actividad (
	curso_id int4 NOT NULL,
	actividad_id int4 NOT NULL,
	fecha_fin timestamptz NULL,
	estado varchar(20) NOT NULL,
	CONSTRAINT historial_curso_actividad_pkey PRIMARY KEY (curso_id, actividad_id),
	CONSTRAINT historial_curso_actividad_actividad_id_fkey FOREIGN KEY (actividad_id) REFERENCES rustdema2.actividades(id) ON DELETE CASCADE,
	CONSTRAINT historial_curso_actividad_curso_id_fkey FOREIGN KEY (curso_id) REFERENCES rustdema2.cursos(id) ON DELETE CASCADE
);


-- rustdema2.personalizacion_portafolio definition

-- Drop table

-- DROP TABLE rustdema2.personalizacion_portafolio;

CREATE TABLE rustdema2.personalizacion_portafolio (
	id serial4 NOT NULL,
	portafolio_id int4 NULL,
	estilos jsonb NULL,
	orden_componentes jsonb NULL,
	privacidad_componentes jsonb NULL,
	CONSTRAINT personalizacion_portafolio_pkey PRIMARY KEY (id),
	CONSTRAINT uq_personalizacion_portafolio UNIQUE (portafolio_id),
	CONSTRAINT personalizacion_portafolio_portafolio_id_fkey FOREIGN KEY (portafolio_id) REFERENCES rustdema2.portafolios(id) ON DELETE CASCADE
);


-- rustdema2.personalizacion_webinar definition

-- Drop table

-- DROP TABLE rustdema2.personalizacion_webinar;

CREATE TABLE rustdema2.personalizacion_webinar (
	id serial4 NOT NULL,
	webinar_id int4 NULL,
	estilos jsonb NULL,
	orden_componentes jsonb NULL,
	privacidad_componentes jsonb NULL,
	CONSTRAINT personalizacion_webinar_pkey PRIMARY KEY (id),
	CONSTRAINT uq_personalizacion_webinar UNIQUE (webinar_id),
	CONSTRAINT personalizacion_webinar_webinar_id_fkey FOREIGN KEY (webinar_id) REFERENCES rustdema2.webinars(id) ON DELETE CASCADE
);


-- rustdema2.portafolio_contenidos definition

-- Drop table

-- DROP TABLE rustdema2.portafolio_contenidos;

CREATE TABLE rustdema2.portafolio_contenidos (
	id serial4 NOT NULL,
	portafolio_id int4 NULL,
	contenido_id int4 NULL,
	CONSTRAINT portafolio_contenidos_pkey PRIMARY KEY (id),
	CONSTRAINT uq_portafolio_contenido UNIQUE (portafolio_id, contenido_id),
	CONSTRAINT portafolio_contenidos_contenido_id_fkey FOREIGN KEY (contenido_id) REFERENCES rustdema2.contenido_transversal(id) ON DELETE CASCADE,
	CONSTRAINT portafolio_contenidos_portafolio_id_fkey FOREIGN KEY (portafolio_id) REFERENCES rustdema2.portafolios(id) ON DELETE CASCADE
);


-- rustdema2.preguntas_evaluacion definition

-- Drop table

-- DROP TABLE rustdema2.preguntas_evaluacion;

CREATE TABLE rustdema2.preguntas_evaluacion (
	id serial4 NOT NULL,
	id_evaluacion int4 NOT NULL,
	id_pregunta int4 NULL,
	texto_pregunta text NOT NULL,
	"tipo_pregunta" rustdema2."tipo_pregunta" NOT NULL,
	opciones jsonb DEFAULT '[]'::jsonb NULL,
	respuesta_correcta jsonb NULL,
	puntaje numeric(5, 2) NOT NULL,
	orden_visualizacion int4 DEFAULT 0 NOT NULL,
	fecha_creacion timestamptz DEFAULT now() NULL,
	fecha_actualizacion timestamptz DEFAULT now() NULL,
	CONSTRAINT preguntas_evaluacion_pkey PRIMARY KEY (id),
	CONSTRAINT preguntas_evaluacion_id_evaluacion_fkey FOREIGN KEY (id_evaluacion) REFERENCES rustdema2.evaluaciones(id) ON DELETE CASCADE,
	CONSTRAINT preguntas_evaluacion_id_pregunta_fkey FOREIGN KEY (id_pregunta) REFERENCES rustdema2.banco_preguntas(id) ON DELETE SET NULL
);
CREATE INDEX idx_preguntas_evaluacion_orden ON rustdema2.preguntas_evaluacion USING btree (id_evaluacion, orden_visualizacion);


-- rustdema2.webinar_modulos definition

-- Drop table

-- DROP TABLE rustdema2.webinar_modulos;

CREATE TABLE rustdema2.webinar_modulos (
	id serial4 NOT NULL,
	webinar_id int4 NULL,
	titulo varchar(200) NOT NULL,
	descripcion text NULL,
	orden int4 DEFAULT 0 NOT NULL,
	tipo_contenido varchar(50) DEFAULT 'video'::character varying NOT NULL,
	contenido_url text NULL,
	duracion_estimada int4 NULL,
	obligatorio bool DEFAULT true NOT NULL,
	fecha_creacion timestamptz DEFAULT now() NULL,
	fecha_actualizacion timestamptz DEFAULT now() NULL,
	CONSTRAINT webinar_modulos_pkey PRIMARY KEY (id),
	CONSTRAINT webinar_modulos_tipo_contenido_check CHECK (((tipo_contenido)::text = ANY ((ARRAY['video'::character varying, 'presentacion'::character varying, 'actividad'::character varying, 'quiz'::character varying])::text[]))),
	CONSTRAINT webinar_modulos_webinar_id_fkey FOREIGN KEY (webinar_id) REFERENCES rustdema2.webinars(id) ON DELETE CASCADE
);
CREATE INDEX idx_webinar_modulos_webinar ON rustdema2.webinar_modulos USING btree (webinar_id);

-- Table Triggers

create trigger actualizar_webinar_modulos_modtime before
update
    on
    rustdema2.webinar_modulos for each row execute function rustdema2.actualizar_fecha_modificacion();


-- rustdema2.webinar_progreso_estudiantes definition

-- Drop table

-- DROP TABLE rustdema2.webinar_progreso_estudiantes;

CREATE TABLE rustdema2.webinar_progreso_estudiantes (
	webinar_id int4 NOT NULL,
	estudiante_id int4 NOT NULL,
	progreso_actual int4 DEFAULT 0 NOT NULL,
	modulos_completados int4 DEFAULT 0 NOT NULL,
	tiempo_total_visto int4 DEFAULT 0 NOT NULL,
	ultima_actividad timestamptz NULL,
	completado bool DEFAULT false NOT NULL,
	fecha_completado timestamptz NULL,
	fecha_creacion timestamptz DEFAULT now() NULL,
	fecha_actualizacion timestamptz DEFAULT now() NULL,
	CONSTRAINT webinar_progreso_estudiantes_pkey PRIMARY KEY (webinar_id, estudiante_id),
	CONSTRAINT webinar_progreso_estudiantes_progreso_actual_check CHECK (((progreso_actual >= 0) AND (progreso_actual <= 100))),
	CONSTRAINT webinar_progreso_estudiantes_estudiante_id_fkey FOREIGN KEY (estudiante_id) REFERENCES rustdema2.usuarios(id) ON DELETE CASCADE,
	CONSTRAINT webinar_progreso_estudiantes_webinar_id_fkey FOREIGN KEY (webinar_id) REFERENCES rustdema2.webinars(id) ON DELETE CASCADE
);
CREATE INDEX idx_webinar_progreso_estudiante ON rustdema2.webinar_progreso_estudiantes USING btree (estudiante_id);


-- rustdema2.contenidos_unidad definition

-- Drop table

-- DROP TABLE rustdema2.contenidos_unidad;

CREATE TABLE rustdema2.contenidos_unidad (
	id serial4 NOT NULL,
	unidad_id int4 NOT NULL,
	tipo_contenido varchar(50) NOT NULL,
	titulo varchar(200) NOT NULL,
	descripcion text NULL,
	orden int4 DEFAULT 0 NOT NULL,
	contenido text NULL,
	url text NULL,
	visible bool DEFAULT true NOT NULL,
	obligatorio bool DEFAULT false NOT NULL,
	puntos int4 NULL,
	fecha_limite timestamptz NULL,
	duracion_estimada int4 NULL,
	examen_id int4 NULL,
	entrega_id int4 NULL,
	fecha_creacion timestamptz DEFAULT now() NULL,
	fecha_actualizacion timestamptz DEFAULT now() NULL,
	CONSTRAINT contenidos_unidad_pkey PRIMARY KEY (id),
	CONSTRAINT contenidos_unidad_tipo_check CHECK (((tipo_contenido)::text = ANY ((ARRAY['texto'::character varying, 'documento'::character varying, 'video'::character varying, 'enlace'::character varying, 'examen'::character varying, 'entrega'::character varying, 'actividad'::character varying])::text[]))),
	CONSTRAINT contenidos_unidad_entrega_id_fkey FOREIGN KEY (entrega_id) REFERENCES rustdema2.entregas(id) ON DELETE CASCADE,
	CONSTRAINT contenidos_unidad_unidad_id_fkey FOREIGN KEY (unidad_id) REFERENCES rustdema2.unidades(id) ON DELETE CASCADE
);
CREATE INDEX idx_contenidos_unidad ON rustdema2.contenidos_unidad USING btree (unidad_id);


-- rustdema2.evaluaciones_calificacion definition

-- Drop table

-- DROP TABLE rustdema2.evaluaciones_calificacion;

CREATE TABLE rustdema2.evaluaciones_calificacion (
	id serial4 NOT NULL,
	evaluacion_id int4 NULL,
	estudiante_id int4 NULL,
	calificacion numeric(5, 2) NULL,
	fecha_registro timestamptz DEFAULT now() NULL,
	CONSTRAINT evaluaciones_calificacion_pkey PRIMARY KEY (id),
	CONSTRAINT uq_evaluacion_estudiante UNIQUE (evaluacion_id, estudiante_id),
	CONSTRAINT evaluaciones_calificacion_estudiante_id_fkey FOREIGN KEY (estudiante_id) REFERENCES rustdema2.usuarios(id) ON DELETE CASCADE,
	CONSTRAINT evaluaciones_calificacion_evaluacion_id_fkey FOREIGN KEY (evaluacion_id) REFERENCES rustdema2.evaluaciones_sesion(id) ON DELETE CASCADE
);

-- rustdema2.experiencia_profesional definition

-- Drop table

-- DROP TABLE rustdema2.experiencia_profesional;

CREATE TABLE rustdema2.experiencia_profesional (
	id serial4 NOT NULL,
	usuario_id int4 NOT NULL,
	empresa varchar(150) NOT NULL,
	cargo varchar(150) NOT NULL,
	descripcion text NULL,
	fecha_inicio date NOT NULL,
	fecha_fin date NULL,
	es_principal bool DEFAULT false NOT NULL,
	estado bool DEFAULT true NOT NULL,
	fecha_creacion timestamptz DEFAULT now() NULL,
	fecha_actualizacion timestamptz DEFAULT now() NULL,
	CONSTRAINT experiencia_profesional_pkey PRIMARY KEY (id),
	CONSTRAINT experiencia_profesional_usuario_id_fkey FOREIGN KEY (usuario_id) REFERENCES rustdema2.usuarios(id) ON DELETE CASCADE
);
CREATE INDEX idx_experiencia_usuario ON rustdema2.experiencia_profesional USING btree (usuario_id);
CREATE INDEX idx_experiencia_principal ON rustdema2.experiencia_profesional USING btree (usuario_id, es_principal) WHERE (es_principal = true);

-- Table Triggers

create trigger actualizar_experiencia_profesional_modtime before
update
    on
    rustdema2.experiencia_profesional for each row execute function rustdema2.actualizar_fecha_modificacion();


-- rustdema2.dias_festivos definition

-- Drop table

-- DROP TABLE rustdema2.dias_festivos;

CREATE TYPE rustdema2.tipo_festivo AS ENUM (
    'nacional',
    'regional',
    'institucional',
    'otro'
);

CREATE TABLE rustdema2.dias_festivos (
	id serial4 NOT NULL,
	nombre varchar(150) NOT NULL,
	descripcion text NULL,
	fecha date NOT NULL,
	tipo rustdema2.tipo_festivo DEFAULT 'nacional' NOT NULL,
	recurrente bool DEFAULT true NOT NULL,
	estado bool DEFAULT true NOT NULL,
	fecha_creacion timestamptz DEFAULT now() NULL,
	fecha_actualizacion timestamptz DEFAULT now() NULL,
	CONSTRAINT dias_festivos_pkey PRIMARY KEY (id),
	CONSTRAINT dias_festivos_fecha_nombre_key UNIQUE (fecha, nombre)
);
CREATE INDEX idx_dias_festivos_fecha ON rustdema2.dias_festivos USING btree (fecha);
CREATE INDEX idx_dias_festivos_tipo ON rustdema2.dias_festivos USING btree (tipo);

-- Table Triggers

create trigger actualizar_dias_festivos_modtime before
update
    on
    rustdema2.dias_festivos for each row execute function rustdema2.actualizar_fecha_modificacion();


-- rustdema2.reportes_errores definition

-- Drop table

-- DROP TABLE rustdema2.reportes_errores;

CREATE TYPE rustdema2.tipo_reporte AS ENUM (
    'error',
    'sugerencia',
    'mejora',
    'otro'
);

CREATE TYPE rustdema2.estado_reporte AS ENUM (
    'recibido',
    'en_revision',
    'en_desarrollo',
    'resuelto',
    'rechazado',
    'cerrado'
);

CREATE TYPE rustdema2.prioridad_reporte AS ENUM (
    'baja',
    'media',
    'alta',
    'critica'
);

CREATE TABLE rustdema2.reportes_errores (
	id serial4 NOT NULL,
	usuario_id int4 NULL,
	titulo varchar(200) NOT NULL,
	descripcion text NOT NULL,
	tipo rustdema2.tipo_reporte DEFAULT 'error' NOT NULL,
	prioridad rustdema2.prioridad_reporte DEFAULT 'media' NOT NULL,
	estado rustdema2.estado_reporte DEFAULT 'recibido' NOT NULL,
	captura_url text NULL,
	responsable_id int4 NULL,
	fecha_resolucion timestamptz NULL,
	solucion text NULL,
	fecha_creacion timestamptz DEFAULT now() NULL,
	fecha_actualizacion timestamptz DEFAULT now() NULL,
	CONSTRAINT reportes_errores_pkey PRIMARY KEY (id),
	CONSTRAINT reportes_errores_responsable_id_fkey FOREIGN KEY (responsable_id) REFERENCES rustdema2.usuarios(id) ON DELETE SET NULL,
	CONSTRAINT reportes_errores_usuario_id_fkey FOREIGN KEY (usuario_id) REFERENCES rustdema2.usuarios(id) ON DELETE SET NULL
);
CREATE INDEX idx_reportes_estado ON rustdema2.reportes_errores USING btree (estado);
CREATE INDEX idx_reportes_usuario ON rustdema2.reportes_errores USING btree (usuario_id);
CREATE INDEX idx_reportes_prioridad ON rustdema2.reportes_errores USING btree (prioridad, estado);

-- Table Triggers

create trigger actualizar_reportes_errores_modtime before
update
    on
    rustdema2.reportes_errores for each row execute function rustdema2.actualizar_fecha_modificacion();


-- rustdema2.seguimiento_reportes definition

-- Drop table

-- DROP TABLE rustdema2.seguimiento_reportes;

CREATE TYPE rustdema2.tipo_seguimiento AS ENUM (
    'cambio_estado',
    'comentario',
    'asignacion',
    'resolucion',
    'reapertura',
    'otro'
);

CREATE TABLE rustdema2.seguimiento_reportes (
	id serial4 NOT NULL,
	reporte_id int4 NOT NULL,
	usuario_id int4 NULL,
	tipo rustdema2.tipo_seguimiento DEFAULT 'comentario' NOT NULL,
	comentario text NOT NULL,
	estado_anterior rustdema2.estado_reporte NULL,
	estado_nuevo rustdema2.estado_reporte NULL,
	fecha_creacion timestamptz DEFAULT now() NULL,
	CONSTRAINT seguimiento_reportes_pkey PRIMARY KEY (id),
	CONSTRAINT seguimiento_reportes_reporte_id_fkey FOREIGN KEY (reporte_id) REFERENCES rustdema2.reportes_errores(id) ON DELETE CASCADE,
	CONSTRAINT seguimiento_reportes_usuario_id_fkey FOREIGN KEY (usuario_id) REFERENCES rustdema2.usuarios(id) ON DELETE SET NULL
);
CREATE INDEX idx_seguimiento_reporte ON rustdema2.seguimiento_reportes USING btree (reporte_id, fecha_creacion DESC);
CREATE INDEX idx_seguimiento_usuario ON rustdema2.seguimiento_reportes USING btree (usuario_id);

-- Table Triggers

create trigger actualizar_seguimiento_reportes_modtime before
update
    on
    rustdema2.seguimiento_reportes for each row execute function rustdema2.actualizar_fecha_modificacion();


-- ============================================================================
-- Datos iniciales del sistema (schema rustdema2)
-- ============================================================================

-- Insertar roles básicos
INSERT INTO rustdema2.roles (nombre) VALUES
    ('Coordinador'),
    ('Profesor'),
    ('Estudiante')
ON CONFLICT (nombre) DO NOTHING;

-- Insertar áreas de conocimiento de ejemplo
INSERT INTO rustdema2.areas_conocimiento (nombre, descripcion, color_etiqueta, estado) VALUES
    ('Matemáticas', 'Área de conocimiento enfocada en ciencias exactas y lógica matemática', '#FF6B6B', true),
    ('Ciencias Naturales', 'Comprende física, química, biología y ciencias de la tierra', '#4ECDC4', true),
    ('Ciencias Sociales', 'Historia, geografía, cívica y estudios sociales', '#45B7D1', true),
    ('Lenguaje y Literatura', 'Comunicación, lectura comprensiva y expresión escrita', '#96CEB4', true),
    ('Tecnología e Informática', 'Herramientas digitales y pensamiento computacional', '#FFEAA7', true),
    ('Artes', 'Expresión artística, música, danza y teatro', '#DDA0DD', true),
    ('Educación Física', 'Desarrollo corporal, deportes y actividad física', '#FFB74D', true)
ON CONFLICT (nombre) DO NOTHING;

-- Insertar usuarios de ejemplo
INSERT INTO rustdema2.usuarios (
    nombre,
    documento_nit,
    correo,
    contrasena,
    foto_url,
    rol_id,
    semestre,
    genero,
    fecha_nacimiento,
    estado
) VALUES
    ('David A. Anduquia', '1234567890', 'david.anduquia@aulavirtual.com', 'admin123', 'https://randomuser.me/api/portraits/men/1.jpg', 1, NULL, 'M', '1980-01-01', true),
    ('María García Profesora', 'PROF456', 'maria.garcia@aulavirtual.com', 'prof123', 'https://randomuser.me/api/portraits/women/1.jpg', 2, NULL, 'F', '1985-05-15', true),
    ('Carlos López Estudiante', 'EST789', 'carlos.lopez@aulavirtual.com', 'est123', 'https://randomuser.me/api/portraits/men/2.jpg', 3, 5, 'M', '2000-10-20', true),
    ('Ana Rodríguez Estudiante', 'EST790', 'ana.rodriguez@aulavirtual.com', 'est123', 'https://randomuser.me/api/portraits/women/2.jpg', 3, 5, 'F', '2001-03-15', true)
ON CONFLICT (documento_nit) DO NOTHING;

-- Insertar cursos de ejemplo
INSERT INTO rustdema2.cursos (
    nombre,
    descripcion,
    fecha_inicio,
    fecha_fin,
    prerequisito,
    coordinador_id,
    semestre,
    periodo,
    anio_pensum,
    area_conocimiento_id
) VALUES
    ('Álgebra Básica', 'Fundamentos de álgebra para educación media', '2025-02-01', '2025-06-30', NULL, 1, 5, '2025-1', 2025, 1),
    ('Física Mecánica', 'Introducción a la física mecánica clásica', '2025-02-01', '2025-06-30', 'Álgebra Básica', 1, 6, '2025-1', 2025, 2)
ON CONFLICT DO NOTHING;

-- Insertar módulos de ejemplo para el curso de Álgebra Básica
INSERT INTO rustdema2.modulos (
    curso_id, nombre, descripcion, orden, tipo, visible, obligatorio, duracion_estimada
) VALUES
    (1, 'Estructura del Curso', 'Organización y navegación del contenido del curso', 1, 'estructura_contenido', true, true, 30),
    (1, 'Taller de Ejercicios', 'Práctica intensiva con ejercicios resueltos', 2, 'taller', true, false, 120),
    (1, 'Evaluación Final', 'Examen final del módulo de álgebra básica', 3, 'evaluacion', true, true, 90)
ON CONFLICT DO NOTHING;

-- Insertar temas de ejemplo
INSERT INTO rustdema2.temas (
    modulo_id, nombre, descripcion, orden, visible
) VALUES
    (1, 'Introducción a la Programación', 'Conceptos básicos y fundamentos de la programación', 1, true),
    (1, 'Estructuras de Control', 'Control del flujo de ejecución', 2, true)
ON CONFLICT DO NOTHING;

-- Insertar unidades de ejemplo
INSERT INTO rustdema2.unidades (
    tema_id, nombre, descripcion, orden, visible
) VALUES
    (1, 'Algoritmos y Lógica', 'Entendiendo el pensamiento algorítmico', 1, true),
    (1, 'Variables y Tipos de Datos', 'Conceptos fundamentales de datos', 2, true),
    (2, 'Condicionales', 'Estructuras if-else y switch', 1, true)
ON CONFLICT DO NOTHING;

-- Insertar webinars de ejemplo
INSERT INTO rustdema2.webinars (
    curso_id, titulo, descripcion, progreso, estado, duracion, modulos
) VALUES
    (1, 'Introducción a la Programación Web', 'Aprende los fundamentos del desarrollo web con ejemplos prácticos', 75, 'en_progreso', '45 min', 8),
    (2, 'Bases de Datos Relacionales', 'Explora el diseño y gestión de bases de datos con SQL', 100, 'completado', '60 min', 12)
ON CONFLICT DO NOTHING;