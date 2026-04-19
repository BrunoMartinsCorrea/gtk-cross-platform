icon_dir = join_paths(get_option('datadir'), 'icons', 'hicolor', 'scalable', 'apps')

install_data(
  'icons/hicolor/scalable/apps/com.example.GtkCrossPlatform.svg',
  install_dir: icon_dir,
)

install_data(
  'com.example.GtkCrossPlatform.desktop',
  install_dir: join_paths(get_option('datadir'), 'applications'),
)

install_data(
  'com.example.GtkCrossPlatform.metainfo.xml',
  install_dir: join_paths(get_option('datadir'), 'metainfo'),
)

gnome.post_install(
  gtk_update_icon_cache: true,
  update_desktop_database: true,
)
