project('gtk-cross-platform', ['vala', 'c'],
  version: '0.1.0',
  meson_version: '>= 0.62.0',
  default_options: [
    'warning_level=2',
    'buildtype=debugoptimized',
  ]
)

app_id   = 'com.example.GtkCrossPlatform'
app_name = 'gtk-cross-platform'

gnome = import('gnome')
i18n  = import('i18n')

config_h = configuration_data()
config_h.set_quoted('APP_ID',          app_id)
config_h.set_quoted('VERSION',         meson.project_version())
config_h.set_quoted('GETTEXT_PACKAGE', app_name)
config_h.set_quoted('PROFILE',         get_option('profile'))
config_h.set_quoted('LOCALEDIR',
  join_paths(get_option('prefix'), get_option('localedir')))
config_h.set_quoted('PKGDATADIR',
  join_paths(get_option('prefix'), get_option('datadir'), app_name))
config_h.set_quoted('SOURCE_DATADIR',
  meson.current_source_dir() / 'data')

configure_file(output: 'config.h', configuration: config_h)

app_deps = [
  dependency('gtk4',         version: '>= 4.12'),
  dependency('libadwaita-1', version: '>= 1.4'),
]

# Exposed to tests/meson.build via parent scope
core_sources = files(
  'src/core/use_cases/greet_use_case.vala',
)

sources = files(
  'src/app.vala',
  'src/window/main_window.vala',
  'src/ports/i_greeting_service.vala',
  'src/infrastructure/greeting/greeting_service.vala',
  'src/infrastructure/logging/app_logger.vala',
) + core_sources

executable(app_name,
  sources,
  dependencies: app_deps,
  vala_args: [
    '--vapidir=' + meson.current_source_dir() / 'vapi',
    '--pkg=config',
  ],
  c_args: ['-include', 'config.h'],
  install: true,
  install_dir: get_option('bindir'),
)

subdir('po')
subdir('data')
subdir('tests')
