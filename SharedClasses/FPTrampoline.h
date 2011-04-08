//
//  FPTrampoline.h
//  IronJump
//
//  Created by Filip Kunc on 6/5/10.
//  For license see LICENSE.TXT
//

#import "FPGameProtocols.h"

@interface FPTrampoline : NSObject <FPGameObject> 
{
	float x, y;
	int widthSegments;
	int animationCounter;
	int textureIndex;
	int textureDirection;
	BOOL isVisible;
}

- (id)initWithWidthSegments:(int)aWidthSegments;

@end