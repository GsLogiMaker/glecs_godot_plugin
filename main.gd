
extends Control

var mutex:= Mutex.new()

var world:= GlWorld.new()

func _init() -> void:
	pass

func _ready() -> void:
	world.start_rest_api()
	var e:= GlEntity.from(9, world)
	var c:= e.get_component(1)
	prints("C", c.get_source_id(), c.get_id())
	prints("C member", c.get_member("sizes"), c.get_member("alignment"))

func _physics_process(delta: float) -> void:
	world.progress(delta)

func _on_run_tests_pressed() -> void:
	get_tree().change_scene_to_file("res://addons/gut/gui/GutRunner.tscn")


func _on_play_asteroids_pressed() -> void:
	get_tree().change_scene_to_file("res://examples/asteroids/asteroids.tscn")


func _on_compose_release_pressed() -> void:
	build_all_releases()


func build_all_releases():
	if not mutex.try_lock():
		return
	await _build_all_releases()
	mutex.unlock()

func _build_all_releases():
	var targets:= [
		# Linux
		"x86_64-unknown-linux-gnu",
		"i686-unknown-linux-gnu",
		"aarch64-unknown-linux-gnu",
		# Windows
		"x86_64-pc-windows-gnu", 
		#"i686-pc-windows-gnu",
		# Mac
		#"x86_64-apple-darwin",
		#"aarch64-apple-darwin",
	]
	targets = [
		"aarch64-unknown-linux-gnu",
	]
	
	# TODO: Install Rust and gcc
	
	match OS.get_name():
		"Windows":
			push_error("TODO")
			breakpoint
		"macOS":
			push_error("TODO")
			breakpoint
		"Linux", "FreeBSD", "NetBSD", "OpenBSD", "BSD":
			print_rich("Installing dependencies ...")
			await cmd("chmod +x ./scripts/install_dependencies_linux.sh")
			await cmd("./scripts/install_dependencies_linux.sh", "Error installing dependency")
		"Android":
			push_error("TODO")
			breakpoint
		"iOS":
			push_error("TODO")
			breakpoint
		"Web":
			push_error("TODO")
			breakpoint
		
	print_rich("Downloading Rust targets ...")
	for target in targets:
		print_rich("    ", target, " ...")
		await cmd(
			"rustup target add %s" % target,
			"Error occured while downloading target \"%s\"" % target,
		)
	
	print_rich("Compiling Glecs ...")
	for target in targets:
		print_rich("    ", target, " ...")
		await cmd(
			"cargo build --manifest-path ./addons/glecs/rust/glecs/Cargo.toml --target %s --features compile_bindings" % target,
			"Error occured while compiling for target \"%s\"" % target,
		)
	
	print_rich("Done!")

func cmd(string:String, err_msg:="", print_out:=false):
	var args:= string.split(" ")
	var thread:= Thread.new()
	thread.start(func():
		var output:= []
		var err:= OS.execute(args[0], args.slice(1), output, true)
		if err != OK:
			if err_msg.length() != 0:
				printerr("\n--- ", err_msg, " ---\n")
			printerr(output[0])
			return
		if print_out:
			print_rich(output[0])
	)
	while thread.is_alive():
		await get_tree().process_frame
	thread.wait_to_finish()
