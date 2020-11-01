public class LookupSwitch {
	static int i = 0;

	public static void main(String[] args) {
		switch (i) {
			case -1:
				throw new IllegalStateException();
			case 0:
				return;
			case 1:
				throw new IllegalStateException();
			case 4:
				throw new IllegalStateException();
			case 94132:
				throw new IllegalStateException();
			default:
				throw new IllegalStateException();
		}
	}
}
