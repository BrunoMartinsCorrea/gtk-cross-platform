test_deps = [
  dependency('glib-2.0'),
  dependency('gobject-2.0'),
]

greet_test = executable('greet-use-case-test',
  files('unit/core/greet_use_case_test.vala') + core_sources,
  dependencies: test_deps,
  vala_args: [
    '--vapidir=' + meson.project_source_root() / 'vapi',
    '--pkg=config',
  ],
  c_args: ['-include', 'config.h'],
)

test('greet-use-case', greet_test)
