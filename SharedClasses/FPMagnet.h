//
//  FPMagnet.h
//  IronJumpLevelEditor
//
//  Created by Filip Kunc on 7/4/10.
//  For license see LICENSE.TXT
//

#import "FPGameProtocols.h"

@interface FPMagnet : NSObject <FPGameObject> 
{
	float x, y;
	int widthSegments;	
	BOOL isVisible;
}

- (id)initWithWidthSegments:(int)aWidthSegments;

@end
